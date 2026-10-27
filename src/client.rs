//! A small client of the PostgreSQL protocol, over plain TCP.
//!
//! The harness talks to a server only through a socket. This client opens a connection, answers the authentication requests and runs simple queries. Replay uses its authentication step, and the commands use it to reset a database and to run one statement.

use std::io::Write as _;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::frame::{self, Frame};
use crate::message::{Dir, Msg, Val};
use crate::scram;

/// Where a server listens and how to log in to it.
#[derive(Clone, Debug)]
pub(crate) struct Target {
    pub(crate) addr: SocketAddr,
    pub(crate) user: String,
    pub(crate) password: String,
    /// Startup parameters that replay and the client add to each connection, such as `rupg.compat_version`.
    pub(crate) params: Vec<(String, String)>,
}

/// The frontend side of an authentication exchange.
#[derive(Debug, Default)]
pub(crate) struct Auth {
    scram: Option<scram::Client>,
    /// The last `Authentication` request of the server, which names the next `p` message.
    pub(crate) last: Option<&'static str>,
}

impl Auth {
    /// Takes one backend message. Returns an error when the server signature of SCRAM is wrong.
    pub(crate) fn see(&mut self, msg: &Msg) -> Result<(), String> {
        if msg.name().starts_with("Authentication") {
            self.last = Some(msg.name());
        }
        if msg.name() == "AuthenticationSASLFinal"
            && let Some(c) = &self.scram
        {
            c.verify(&String::from_utf8_lossy(msg.vals[0].bytes().unwrap_or_default()))?;
        }
        Ok(())
    }

    /// The first SCRAM message, for a new exchange with a new nonce.
    pub(crate) fn sasl_initial(&mut self, password: &str) -> Msg {
        let c = scram::Client::new(password, None);
        let data = c.first().into_bytes();
        self.scram = Some(c);
        Msg::new(Dir::F, "SASLInitialResponse", vec![str_val("SCRAM-SHA-256"), Val::Str(data)])
    }

    /// The second SCRAM message, from the server-first message.
    pub(crate) fn sasl_response(&mut self, server_first: &[u8]) -> Result<Msg, String> {
        let c = self.scram.as_mut().ok_or("SASLResponse before SASLInitialResponse")?;
        let last = c.last(&String::from_utf8_lossy(server_first))?;
        Ok(Msg::new(Dir::F, "SASLResponse", vec![Val::Str(last.into_bytes())]))
    }

    /// The answer to an authentication request, or None when the request needs no answer.
    pub(crate) fn answer(&mut self, msg: &Msg, password: &str) -> Result<Option<Msg>, String> {
        match msg.name() {
            "AuthenticationCleartextPassword" => {
                Ok(Some(Msg::new(Dir::F, "PasswordMessage", vec![str_val(password)])))
            }
            "AuthenticationSASL" => {
                let offered =
                    msg.vals[0].list().iter().any(|m| m.bytes() == Some(b"SCRAM-SHA-256"));
                if !offered {
                    return Err("the server does not offer SCRAM-SHA-256".into());
                }
                Ok(Some(self.sasl_initial(password)))
            }
            "AuthenticationSASLContinue" => {
                Ok(Some(self.sasl_response(msg.vals[0].bytes().unwrap_or_default())?))
            }
            "AuthenticationMD5Password"
            | "AuthenticationGSS"
            | "AuthenticationSSPI"
            | "AuthenticationKerberosV5" => Err(format!("{} is not supported", msg.name())),
            _ => Ok(None),
        }
    }
}

pub(crate) fn str_val(s: &str) -> Val {
    Val::Str(s.as_bytes().to_vec())
}

/// The startup message of protocol 3.0, with the parameters of the target.
pub(crate) fn startup(target: &Target, database: &str) -> Msg {
    let pairs =
        vec![str_val("user"), str_val(&target.user), str_val("database"), str_val(database)];
    let mut msg = Msg::new(Dir::F, "StartupMessage", vec![Val::Int(196_608), Val::List(pairs)]);
    add_params(&mut msg, &target.params);
    msg
}

/// Adds startup parameters to a `StartupMessage` that does not have them yet.
pub(crate) fn add_params(msg: &mut Msg, params: &[(String, String)]) {
    if let Some(Val::List(pairs)) = msg.vals.get_mut(1) {
        for (k, v) in params {
            if !pairs.chunks(2).any(|p| p[0].bytes() == Some(k.as_bytes())) {
                pairs.push(str_val(k));
                pairs.push(str_val(v));
            }
        }
    }
}

/// One open connection.
#[derive(Debug)]
pub(crate) struct Conn {
    stream: TcpStream,
    /// The `ParameterStatus` messages of the startup, in the order of the server.
    pub(crate) status: Vec<(String, String)>,
}

impl Conn {
    pub(crate) fn connect(target: &Target, database: &str) -> Result<Conn, String> {
        let stream = TcpStream::connect_timeout(&target.addr, Duration::from_secs(10))
            .map_err(|e| format!("connect to {}: {e}", target.addr))?;
        stream.set_nodelay(true).map_err(|e| e.to_string())?;
        stream.set_read_timeout(Some(Duration::from_secs(300))).map_err(|e| e.to_string())?;
        let mut c = Conn { stream, status: Vec::new() };
        c.send(&startup(target, database))?;
        let mut auth = Auth::default();
        loop {
            let msg = c.recv()?;
            auth.see(&msg)?;
            if let Some(answer) = auth.answer(&msg, &target.password)? {
                c.send(&answer)?;
            }
            match msg.name() {
                "ErrorResponse" => return Err(error_text(&msg)),
                "ReadyForQuery" => return Ok(c),
                "ParameterStatus" => {
                    let text = |v: &Val| {
                        String::from_utf8_lossy(v.bytes().unwrap_or_default()).into_owned()
                    };
                    c.status.push((text(&msg.vals[0]), text(&msg.vals[1])));
                }
                _ => {}
            }
        }
    }

    pub(crate) fn send(&mut self, msg: &Msg) -> Result<(), String> {
        msg.encode().write_to(&mut self.stream).map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())
    }

    pub(crate) fn recv(&mut self) -> Result<Msg, String> {
        match frame::read_typed(&mut self.stream) {
            Ok(Some(f)) => Ok(Msg::decode(Dir::B, &f, "")),
            Ok(None) => Err("the server closed the connection".into()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Runs a simple query and returns the backend messages up to `ReadyForQuery`.
    pub(crate) fn query(&mut self, sql: &str) -> Result<Vec<Msg>, String> {
        self.send(&Msg::new(Dir::F, "Query", vec![str_val(sql)]))?;
        let mut out = Vec::new();
        loop {
            let msg = self.recv()?;
            let done = msg.name() == "ReadyForQuery";
            out.push(msg);
            if done {
                return Ok(out);
            }
        }
    }

    /// Runs a simple query and fails on an error.
    pub(crate) fn execute(&mut self, sql: &str) -> Result<(), String> {
        match self.query(sql)?.iter().find(|m| m.name() == "ErrorResponse") {
            Some(e) => Err(format!("{sql}: {}", error_text(e))),
            None => Ok(()),
        }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        let _ = Frame::Typed(b'X', Vec::new()).write_to(&mut self.stream);
    }
}

/// The names of the databases of a server that allow connections, in name order.
pub(crate) fn databases(target: &Target) -> Result<Vec<String>, String> {
    let mut c = Conn::connect(target, "template1")?;
    let sql = "SELECT datname FROM pg_catalog.pg_database WHERE datallowconn ORDER BY datname";
    let rows = c.query(sql)?;
    if let Some(e) = rows.iter().find(|m| m.name() == "ErrorResponse") {
        return Err(format!("{sql}: {}", error_text(e)));
    }
    Ok(rows
        .iter()
        .filter(|m| m.name() == "DataRow")
        .filter_map(|m| m.vals[0].list().first()?.bytes())
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .collect())
}

/// Makes each database of `oracle` that `target` does not have, so that a query of `pg_database` gives the same rows on the two servers (spec/05 section 5.2).
pub(crate) fn same_databases(oracle: &Target, target: &Target) -> Result<(), String> {
    let have = databases(target)?;
    let mut c = Conn::connect(target, "template1")?;
    for name in databases(oracle)?.iter().filter(|n| !have.contains(n)) {
        let quoted = format!("\"{}\"", name.replace('"', "\"\""));
        c.execute(&format!("CREATE DATABASE {quoted} TEMPLATE template0"))?;
    }
    Ok(())
}

/// Makes the databases of `target` the same as `names`: it makes each database that is missing and drops each other database, but not `postgres` and `template1`. A trace keeps the databases of the oracle when the recording started, and a client can list them.
pub(crate) fn set_databases(target: &Target, names: &[&str]) -> Result<(), String> {
    let have = databases(target)?;
    let mut c = Conn::connect(target, "template1")?;
    for name in names.iter().filter(|n| !have.iter().any(|h| h == *n)) {
        let quoted = format!("\"{}\"", name.replace('"', "\"\""));
        c.execute(&format!("CREATE DATABASE {quoted} TEMPLATE template0"))?;
    }
    for name in have.iter().filter(|h| !names.contains(&h.as_str())) {
        if ["postgres", "template1"].contains(&name.as_str()) {
            continue;
        }
        let quoted = format!("\"{}\"", name.replace('"', "\"\""));
        c.execute(&format!("DROP DATABASE {quoted} WITH (FORCE)"))?;
    }
    Ok(())
}

/// The severity, the SQLSTATE and the message of an `ErrorResponse`.
pub(crate) fn error_text(msg: &Msg) -> String {
    let fields = msg.vals.first().map(Val::list).unwrap_or_default();
    let get = |code: u8| {
        fields
            .chunks(2)
            .find(|p| p[0] == Val::Byte(code))
            .and_then(|p| p.get(1)?.bytes())
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default()
    };
    format!("{} {} {}", get(b'S'), get(b'C'), get(b'M'))
}

/// Drops and creates a database, so a run starts from the same state.
pub(crate) fn reset_database(target: &Target, name: &str) -> Result<(), String> {
    if ["postgres", "template0", "template1"].contains(&name) {
        return Err(format!("the harness does not reset the database {name}"));
    }
    let mut c = Conn::connect(target, "template1")?;
    let quoted = format!("\"{}\"", name.replace('"', "\"\""));
    c.execute(&format!("DROP DATABASE IF EXISTS {quoted} WITH (FORCE)"))?;
    c.execute(&format!("CREATE DATABASE {quoted} TEMPLATE template0"))
}
