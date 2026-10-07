//! The recording proxy (spec/21 section 21.3.5).
//!
//! The proxy sits between a client and a server. It forwards every message in both directions without a change and writes each one to a trace, one message on each line. Each client connection gets the next connection number. A `CancelRequest` arrives on its own connection, so it gets its own number too.
//!
//! The proxy does not speak TLS. It answers `SSLRequest` and `GSSENCRequest` itself with `N`, so the client continues without encryption and the proxy can read the messages. It writes the request and its answer to the trace as comments, because the server never sees them. A client with `sslmode=require` cannot record through the proxy.

use std::io::{self, BufWriter, Write as _};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::frame::{self, FrontendState, GSSENC_REQUEST, SSL_REQUEST};
use crate::message::{Dir, Msg, password_name};
use crate::trace::{Event, Line, Writer};

type Sink = Arc<Mutex<Writer<BufWriter<std::fs::File>>>>;

/// A running proxy.
#[derive(Debug)]
pub(crate) struct Proxy {
    pub(crate) addr: SocketAddr,
    stop: Arc<AtomicBool>,
    accept: Option<JoinHandle<Vec<JoinHandle<()>>>>,
    sink: Sink,
}

impl Proxy {
    /// Listens on `listen` and forwards each connection to `server`. Writes the trace to `out` with `header` as its first comments.
    pub(crate) fn start(
        listen: &str,
        server: SocketAddr,
        out: &std::path::Path,
        header: &[String],
    ) -> Result<Proxy, String> {
        let file = std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
        let sink: Sink = Arc::new(Mutex::new(
            Writer::new(BufWriter::new(file), header).map_err(|e| e.to_string())?,
        ));
        let listener = TcpListener::bind(listen).map_err(|e| format!("listen on {listen}: {e}"))?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let next = Arc::new(AtomicU32::new(1));
        let (stop2, sink2) = (stop.clone(), sink.clone());
        let accept = thread::spawn(move || {
            let mut sessions = Vec::new();
            while !stop2.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((client, _)) => {
                        let conn = next.fetch_add(1, Ordering::SeqCst);
                        let sink = sink2.clone();
                        sessions.push(thread::spawn(move || {
                            if let Err(e) = session(conn, client, server, &sink) {
                                note(&sink, &format!("connection {conn}: {e}"));
                            }
                        }));
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => thread::sleep(Duration::from_millis(20)),
                }
            }
            sessions
        });
        Ok(Proxy { addr, stop, accept: Some(accept), sink })
    }

    /// Stops accepting connections, waits for the open ones to end, and flushes the trace.
    pub(crate) fn finish(mut self) -> Result<(), String> {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(accept) = self.accept.take() {
            for s in accept.join().unwrap_or_default() {
                let _ = s.join();
            }
        }
        self.sink
            .lock()
            .map_err(|_| "the trace lock is poisoned".to_string())?
            .flush()
            .map_err(|e| e.to_string())
    }
}

fn write(sink: &Sink, line: &Line) {
    if let Ok(mut w) = sink.lock() {
        let _ = w.line(line);
        let _ = w.flush();
    }
}

fn note(sink: &Sink, text: &str) {
    if let Ok(mut w) = sink.lock() {
        let _ = w.comment(text);
    }
}

/// Forwards one client connection and records it.
fn session(conn: u32, client: TcpStream, server: SocketAddr, sink: &Sink) -> io::Result<()> {
    client.set_nonblocking(false)?;
    client.set_nodelay(true)?;
    let mut from_client = client.try_clone()?;
    let mut to_client = client;
    let mut state = FrontendState::default();
    // The encryption requests come before the server sees anything, so the proxy answers them here.
    let first = loop {
        let Some(frame) = state.read(&mut from_client)? else { return Ok(()) };
        match frame.code() {
            Some(code @ (SSL_REQUEST | GSSENC_REQUEST)) => {
                let name = if code == SSL_REQUEST { "SSLRequest" } else { "GSSENCRequest" };
                note(sink, &format!("{conn} F {name}: the proxy answered N"));
                to_client.write_all(b"N")?;
            }
            _ => break frame,
        }
    };
    let upstream = TcpStream::connect(server)?;
    upstream.set_nodelay(true)?;
    let mut to_server = upstream.try_clone()?;
    let mut from_server = upstream;
    // The type of a `p` message depends on the last authentication request of the server.
    let last_auth: Arc<Mutex<Option<&'static str>>> = Arc::new(Mutex::new(None));
    let backend = {
        let (sink, last_auth) = (sink.clone(), last_auth.clone());
        let mut to_client = to_client.try_clone()?;
        thread::spawn(move || -> io::Result<()> {
            while let Ok(Some(frame)) = frame::read_typed(&mut from_server) {
                let msg = Msg::decode(Dir::B, &frame, "");
                if msg.name().starts_with("Authentication") {
                    *last_auth.lock().map_err(|_| io::ErrorKind::Other)? = Some(msg.name());
                }
                write(&sink, &Line { conn, dir: Dir::B, event: Event::Msg(msg) });
                if frame.write_to(&mut to_client).is_err() {
                    break;
                }
            }
            write(&sink, &Line { conn, dir: Dir::B, event: Event::End });
            let _ = to_client.shutdown(Shutdown::Write);
            Ok(())
        })
    };
    let mut frame = Some(first);
    while let Some(f) = frame {
        let password = password_name(*last_auth.lock().map_err(|_| io::ErrorKind::Other)?);
        let msg = Msg::decode(Dir::F, &f, password);
        write(sink, &Line { conn, dir: Dir::F, event: Event::Msg(msg) });
        if f.write_to(&mut to_server).is_err() {
            break;
        }
        // A client that resets the connection ends the stream like a client that closes it.
        frame = state.read(&mut from_client).unwrap_or(None);
    }
    write(sink, &Line { conn, dir: Dir::F, event: Event::End });
    let _ = to_server.shutdown(Shutdown::Write);
    backend.join().map_err(|_| io::Error::other("the backend thread panicked"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, read_typed, read_untyped};
    use std::io::Read as _;

    /// A server that answers one startup with `AuthenticationOk` and `ReadyForQuery`, one query with `CommandComplete`, and closes on `Terminate`.
    fn fake_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut s, _) = listener.accept().unwrap();
            read_untyped(&mut s).unwrap().unwrap();
            let mut out = Frame::Typed(b'R', 0i32.to_be_bytes().to_vec()).to_bytes();
            out.extend(Frame::Typed(b'Z', b"I".to_vec()).to_bytes());
            s.write_all(&out).unwrap();
            while let Some(Frame::Typed(tag, _)) = read_typed(&mut s).unwrap() {
                if tag == b'X' {
                    break;
                }
                let mut out = Frame::Typed(b'C', b"SELECT 1\0".to_vec()).to_bytes();
                out.extend(Frame::Typed(b'Z', b"I".to_vec()).to_bytes());
                s.write_all(&out).unwrap();
            }
        });
        addr
    }

    #[test]
    fn the_proxy_forwards_and_records_both_directions() {
        let dir = std::env::temp_dir().join(format!("rupg-compat-proxy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("t.trace");
        let proxy = Proxy::start("127.0.0.1:0", fake_server(), &out, &["test".into()]).unwrap();
        let mut c = TcpStream::connect(proxy.addr).unwrap();
        c.write_all(&Frame::Untyped(SSL_REQUEST.to_be_bytes().to_vec()).to_bytes()).unwrap();
        let mut answer = [0u8; 1];
        c.read_exact(&mut answer).unwrap();
        assert_eq!(&answer, b"N");
        let mut startup = 196_608i32.to_be_bytes().to_vec();
        startup.extend_from_slice(b"user\0postgres\0\0");
        c.write_all(&Frame::Untyped(startup).to_bytes()).unwrap();
        assert_eq!(read_typed(&mut c).unwrap(), Some(Frame::Typed(b'R', vec![0, 0, 0, 0])));
        assert_eq!(read_typed(&mut c).unwrap(), Some(Frame::Typed(b'Z', b"I".to_vec())));
        c.write_all(&Frame::Typed(b'Q', b"SELECT 1\0".to_vec()).to_bytes()).unwrap();
        assert_eq!(read_typed(&mut c).unwrap(), Some(Frame::Typed(b'C', b"SELECT 1\0".to_vec())));
        assert_eq!(read_typed(&mut c).unwrap(), Some(Frame::Typed(b'Z', b"I".to_vec())));
        c.write_all(&Frame::Typed(b'X', vec![]).to_bytes()).unwrap();
        assert_eq!(read_typed(&mut c).unwrap(), None);
        drop(c);
        proxy.finish().unwrap();
        let text = std::fs::read_to_string(&out).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        // The two directions run in two threads, so only the order within one direction is fixed.
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "# test");
        assert_eq!(lines[1], "# 1 F SSLRequest: the proxy answered N");
        let f: Vec<&str> = lines.iter().copied().filter(|l| l.starts_with("1 F")).collect();
        let b: Vec<&str> = lines.iter().copied().filter(|l| l.starts_with("1 B")).collect();
        assert_eq!(
            f,
            [
                "1 F StartupMessage 3.0 \"user\" \"postgres\"",
                "1 F Query \"SELECT 1\"",
                "1 F Terminate",
                "1 F End"
            ]
        );
        assert_eq!(
            b,
            [
                "1 B AuthenticationOk",
                "1 B ReadyForQuery I",
                "1 B CommandComplete \"SELECT 1\"",
                "1 B ReadyForQuery I",
                "1 B End"
            ]
        );
        assert_eq!(lines.len(), 11);
    }
}
