//! The framing of the PostgreSQL v3 protocol: how to cut a byte stream into messages.
//!
//! A frontend starts with untyped packets (an `Int32` length that counts itself, then the body) until it sends a `StartupMessage`. After that, every message in both directions is typed: one tag byte, an `Int32` length that counts itself but not the tag, then the body. The answer to `SSLRequest` and `GSSENCRequest` is one byte with no length.

use std::io::{self, Read, Write};

/// The request codes of the untyped frontend packets.
pub(crate) const CANCEL_REQUEST: i32 = 80_877_102;
pub(crate) const SSL_REQUEST: i32 = 80_877_103;
pub(crate) const GSSENC_REQUEST: i32 = 80_877_104;

/// The largest message that the harness accepts. PostgreSQL limits a field to 1 GB.
const MAX_LEN: usize = 1 << 30;

/// One frame on the wire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Frame {
    /// An untyped frontend packet. The body starts with the protocol version or the request code.
    Untyped(Vec<u8>),
    /// A typed message: the tag and the body without the length.
    Typed(u8, Vec<u8>),
    /// The one-byte answer to `SSLRequest` or `GSSENCRequest`.
    Byte(u8),
}

impl Frame {
    /// The request code or the protocol version of an untyped packet.
    pub(crate) fn code(&self) -> Option<i32> {
        match self {
            Frame::Untyped(body) if body.len() >= 4 => {
                Some(i32::from_be_bytes([body[0], body[1], body[2], body[3]]))
            }
            _ => None,
        }
    }

    /// The bytes of the frame on the wire.
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        match self {
            Frame::Untyped(body) => {
                let mut out = Vec::with_capacity(body.len() + 4);
                out.extend_from_slice(&len32(body.len() + 4).to_be_bytes());
                out.extend_from_slice(body);
                out
            }
            Frame::Typed(tag, body) => {
                let mut out = Vec::with_capacity(body.len() + 5);
                out.push(*tag);
                out.extend_from_slice(&len32(body.len() + 4).to_be_bytes());
                out.extend_from_slice(body);
                out
            }
            Frame::Byte(b) => vec![*b],
        }
    }

    pub(crate) fn write_to(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.to_bytes())
    }
}

fn len32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// Reads one untyped packet. Returns `None` at a clean end of the stream.
pub(crate) fn read_untyped(r: &mut impl Read) -> io::Result<Option<Frame>> {
    let mut len = [0u8; 4];
    if !read_or_eof(r, &mut len)? {
        return Ok(None);
    }
    let body = read_body(r, i32::from_be_bytes(len))?;
    Ok(Some(Frame::Untyped(body)))
}

/// Reads one typed message. Returns `None` at a clean end of the stream.
pub(crate) fn read_typed(r: &mut impl Read) -> io::Result<Option<Frame>> {
    let mut head = [0u8; 5];
    if !read_or_eof(r, &mut head[..1])? {
        return Ok(None);
    }
    r.read_exact(&mut head[1..])?;
    let body = read_body(r, i32::from_be_bytes([head[1], head[2], head[3], head[4]]))?;
    Ok(Some(Frame::Typed(head[0], body)))
}

fn read_body(r: &mut impl Read, len: i32) -> io::Result<Vec<u8>> {
    let len =
        usize::try_from(len).ok().filter(|&n| (4..=MAX_LEN).contains(&n)).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("bad message length {len}"))
        })?;
    let mut body = vec![0u8; len - 4];
    r.read_exact(&mut body)?;
    Ok(body)
}

/// Fills `buf`. Returns false when the stream ends before the first byte.
fn read_or_eof(r: &mut impl Read, buf: &mut [u8]) -> io::Result<bool> {
    let mut done = 0;
    while done < buf.len() {
        match r.read(&mut buf[done..]) {
            Ok(0) if done == 0 => return Ok(false),
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => done += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

/// Follows the state of the frontend side: untyped packets until the `StartupMessage`, typed messages after it.
#[derive(Debug, Default)]
pub(crate) struct FrontendState {
    started: bool,
}

impl FrontendState {
    pub(crate) fn read(&mut self, r: &mut impl Read) -> io::Result<Option<Frame>> {
        if self.started {
            return read_typed(r);
        }
        let frame = read_untyped(r)?;
        if let Some(code) = frame.as_ref().and_then(Frame::code) {
            // A protocol version has the major version 3 in the high 16 bits. The request codes have 1234.
            self.started = code >> 16 == 3;
        }
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip() {
        for frame in [
            Frame::Untyped(SSL_REQUEST.to_be_bytes().to_vec()),
            Frame::Typed(b'Q', b"SELECT 1\0".to_vec()),
            Frame::Typed(b'S', vec![]),
        ] {
            let bytes = frame.to_bytes();
            let mut r = bytes.as_slice();
            let back = match frame {
                Frame::Untyped(_) => read_untyped(&mut r),
                _ => read_typed(&mut r),
            };
            assert_eq!(back.unwrap(), Some(frame));
            assert!(r.is_empty());
        }
    }

    #[test]
    fn the_frontend_state_switches_after_the_startup_message() {
        let mut bytes = Frame::Untyped(SSL_REQUEST.to_be_bytes().to_vec()).to_bytes();
        let mut startup = 196_608i32.to_be_bytes().to_vec();
        startup.extend_from_slice(b"user\0u\0\0");
        bytes.extend(Frame::Untyped(startup).to_bytes());
        bytes.extend(Frame::Typed(b'X', vec![]).to_bytes());
        let mut r = bytes.as_slice();
        let mut state = FrontendState::default();
        assert_eq!(state.read(&mut r).unwrap().unwrap().code(), Some(SSL_REQUEST));
        assert_eq!(state.read(&mut r).unwrap().unwrap().code(), Some(196_608));
        assert_eq!(state.read(&mut r).unwrap(), Some(Frame::Typed(b'X', vec![])));
        assert_eq!(state.read(&mut r).unwrap(), None);
    }

    #[test]
    fn bad_lengths_and_truncation_are_errors() {
        let mut r: &[u8] = &[b'Q', 0, 0, 0, 3];
        assert!(read_typed(&mut r).is_err());
        let mut r: &[u8] = &[b'Q', 0, 0, 0, 9, b'x'];
        assert!(read_typed(&mut r).is_err());
        let mut r: &[u8] = &[b'Q', 0];
        assert!(read_typed(&mut r).is_err());
    }
}
