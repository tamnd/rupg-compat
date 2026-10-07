//! Replay: send the frontend side of a trace to a server and record what it answers (spec/21 section 21.3.5).
//!
//! Replay keeps the order of the lines of each connection. Before it sends a frontend message, it waits for the backend messages of the same connection that come before that message in the trace. It counts the messages after which a server waits for the client (`ReadyForQuery`, `CopyInResponse`, `CopyBothResponse` and the authentication requests) and waits for the same number. A group with none of them, for example the answers to a `Flush`, waits for the same number of messages, or until the server is quiet.
//!
//! Replay solves the three problems of section 21.3.5:
//!
//! 1. OIDs differ. Replay learns a map from the OIDs of the trace to the OIDs of the server. It reads them from the table and type OIDs of `RowDescription`, the types of `ParameterDescription` and the columns of type `oid` in `DataRow`, for user objects only (OID 16384 and up). It rewrites the OIDs in `Query`, `Parse`, `Bind` and `FunctionCall` before it sends them, and maps the answers back to the OIDs of the trace.
//! 2. SCRAM has new nonces each time. Replay makes a new SCRAM exchange with each server.
//! 3. Cancel keys differ. Replay rewrites each `CancelRequest` to the process ID and the key that the server sent on the connection that the trace cancels. The key keeps the length that the server sent, so it works for protocol 3.0 and 3.2.

use std::collections::{BTreeMap, VecDeque};
use std::net::{Shutdown, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::client::{Auth, Target, add_params, str_val};
use crate::frame;
use crate::message::{Dir, Msg, Val};
use crate::trace::{Event, Line, Trace};

/// The first OID that PostgreSQL gives to a user object.
pub(crate) const FIRST_NORMAL_OID: u32 = 16_384;

/// The type OID of `oid`.
const OID_TYPE: i64 = 26;

/// How long replay waits for a server that is quiet after a group without a terminator.
const IDLE: Duration = Duration::from_millis(500);

/// The map between the OIDs of a trace and the OIDs of one server.
#[derive(Debug, Default)]
pub(crate) struct OidMap {
    to_server: BTreeMap<u32, u32>,
    to_trace: BTreeMap<u32, u32>,
    /// How many values replay changed with the map, in both directions.
    pub(crate) applied: usize,
}

impl OidMap {
    pub(crate) fn len(&self) -> usize {
        self.to_server.len()
    }

    /// Spec/05 section 5.3 item 2 excludes the values of user OIDs but compares the gaps between consecutive ones. Returns the number of pairs of consecutive user OIDs of the trace and the number of pairs with another gap on the server.
    pub(crate) fn gaps(&self) -> (usize, usize) {
        let pairs: Vec<(i64, i64)> =
            self.to_server.iter().map(|(&t, &s)| (i64::from(t), i64::from(s))).collect();
        let differ = pairs.windows(2).filter(|w| w[1].0 - w[0].0 != w[1].1 - w[0].1).count();
        (pairs.len().saturating_sub(1), differ)
    }

    /// Learns that `trace` in the trace is `server` on the server. The first pair for an OID wins.
    pub(crate) fn learn(&mut self, trace: u32, server: u32) {
        if trace < FIRST_NORMAL_OID || server < FIRST_NORMAL_OID {
            return;
        }
        if self.to_server.contains_key(&trace) || self.to_trace.contains_key(&server) {
            return;
        }
        self.to_server.insert(trace, server);
        self.to_trace.insert(server, trace);
    }

    fn map(&mut self, oid: u32, back: bool) -> u32 {
        let table = if back { &self.to_trace } else { &self.to_server };
        match table.get(&oid) {
            Some(&o) if o != oid => {
                self.applied += 1;
                o
            }
            _ => oid,
        }
    }

    /// Maps an OID in an integer field.
    fn map_int(&mut self, v: &mut Val, back: bool) {
        if let Val::Int(i) = v
            && let Ok(oid) = u32::try_from(*i).or_else(|_| i32::try_from(*i).map(|x| x as u32))
        {
            let m = self.map(oid, back);
            if m != oid {
                *i = if *i < 0 { i64::from(m as i32) } else { i64::from(m) };
            }
        }
    }

    /// Maps a value that is an OID in text, like a column of type `oid`.
    fn map_text_value(&mut self, v: &mut Val, back: bool) {
        if let Val::Str(s) = v
            && let Some(oid) = parse_oid(s)
        {
            let m = self.map(oid, back);
            if m != oid {
                *s = m.to_string().into_bytes();
            }
        }
    }

    /// Maps each number in a statement that is a mapped OID. A number counts only when no letter, digit, `_` or `.` touches it.
    pub(crate) fn map_sql(&mut self, sql: &[u8], back: bool) -> Vec<u8> {
        let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b >= 0x80;
        let mut out = Vec::with_capacity(sql.len());
        let mut i = 0;
        while i < sql.len() {
            if sql[i].is_ascii_digit() && (i == 0 || !word(sql[i - 1])) {
                let end =
                    sql[i..].iter().position(|b| !b.is_ascii_digit()).map_or(sql.len(), |n| i + n);
                let digits = &sql[i..end];
                if (end == sql.len() || !word(sql[end]))
                    && let Some(oid) = parse_oid(digits)
                {
                    out.extend_from_slice(self.map(oid, back).to_string().as_bytes());
                    i = end;
                    continue;
                }
                out.extend_from_slice(digits);
                i = end;
                continue;
            }
            out.push(sql[i]);
            i += 1;
        }
        out
    }
}

fn parse_oid(s: &[u8]) -> Option<u32> {
    if s.is_empty()
        || s.len() > 10
        || !s.iter().all(u8::is_ascii_digit)
        || (s.len() > 1 && s[0] == b'0')
    {
        return None;
    }
    std::str::from_utf8(s).ok()?.parse().ok()
}

/// The types of the columns of a `RowDescription`.
fn row_types(msg: &Msg) -> Vec<i64> {
    msg.vals[0].list().iter().map(|f| f.list().get(3).and_then(Val::int).unwrap_or(0)).collect()
}

/// Learns the OIDs of one pair of messages: `trace` from the trace and `server` from the server.
fn learn(map: &mut OidMap, trace: &Msg, server: &Msg, types: &[i64]) {
    if trace.name() != server.name() {
        return;
    }
    let ints = |v: &Val, i: usize| v.list().get(i).and_then(Val::int).map(|x| x as u32);
    match server.name() {
        "RowDescription" => {
            for (t, s) in trace.vals[0].list().iter().zip(server.vals[0].list()) {
                for i in [1, 3] {
                    if let (Some(a), Some(b)) = (ints(t, i), ints(s, i)) {
                        map.learn(a, b);
                    }
                }
            }
        }
        "ParameterDescription" => {
            for (t, s) in trace.vals[0].list().iter().zip(server.vals[0].list()) {
                if let (Some(a), Some(b)) = (t.int(), s.int()) {
                    map.learn(a as u32, b as u32);
                }
            }
        }
        "DataRow" => {
            let cols = trace.vals[0].list().iter().zip(server.vals[0].list());
            for ((t, s), ty) in cols.zip(types) {
                if *ty == OID_TYPE
                    && let (Some(a), Some(b)) =
                        (t.bytes().and_then(parse_oid), s.bytes().and_then(parse_oid))
                {
                    map.learn(a, b);
                }
            }
        }
        _ => {}
    }
}

/// Maps the OIDs of a backend message. `back` maps from the server to the trace.
fn map_backend(map: &mut OidMap, msg: &mut Msg, types: &[i64]) {
    match msg.name() {
        "RowDescription" => {
            if let Val::List(fields) = &mut msg.vals[0] {
                for f in fields {
                    if let Val::Tuple(v) = f {
                        map.map_int(&mut v[1], true);
                        map.map_int(&mut v[3], true);
                    }
                }
            }
        }
        "ParameterDescription" => {
            if let Val::List(types) = &mut msg.vals[0] {
                for t in types {
                    map.map_int(t, true);
                }
            }
        }
        "DataRow" => {
            if let Val::List(cols) = &mut msg.vals[0] {
                for (c, ty) in cols.iter_mut().zip(types) {
                    if *ty == OID_TYPE {
                        map.map_text_value(c, true);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Maps the OIDs of a frontend message to the server.
fn map_frontend(map: &mut OidMap, msg: &mut Msg) {
    match msg.name() {
        "Query" => {
            if let Val::Str(s) = &mut msg.vals[0] {
                *s = map.map_sql(s, false);
            }
        }
        "Parse" => {
            if let Val::Str(s) = &mut msg.vals[1] {
                *s = map.map_sql(s, false);
            }
            if let Val::List(types) = &mut msg.vals[2] {
                for t in types {
                    map.map_int(t, false);
                }
            }
        }
        "Bind" => {
            let formats: Vec<i64> = msg.vals[2].list().iter().filter_map(Val::int).collect();
            if let Val::List(params) = &mut msg.vals[3] {
                for (i, p) in params.iter_mut().enumerate() {
                    let format = match formats.len() {
                        0 => 0,
                        1 => formats[0],
                        _ => formats.get(i).copied().unwrap_or(0),
                    };
                    if format == 0 {
                        map.map_text_value(p, false);
                    }
                }
            }
        }
        "FunctionCall" => map.map_int(&mut msg.vals[0], false),
        _ => {}
    }
}

/// A message after which the server waits for the client.
fn is_terminator(msg: &Msg) -> bool {
    matches!(
        msg.name(),
        "ReadyForQuery"
            | "CopyInResponse"
            | "CopyBothResponse"
            | "AuthenticationCleartextPassword"
            | "AuthenticationMD5Password"
            | "AuthenticationSASL"
            | "AuthenticationSASLContinue"
            | "AuthenticationGSS"
            | "AuthenticationGSSContinue"
            | "AuthenticationSSPI"
    )
}

fn key_of(msg: &Msg) -> Option<(i64, Vec<u8>)> {
    Some((msg.vals.first()?.int()?, msg.vals.get(1)?.bytes()?.to_vec()))
}

/// What one replay did.
#[derive(Debug, Default)]
pub(crate) struct Replayed {
    /// The frontend lines of the trace and the backend lines of the server, with the OIDs mapped back to the trace.
    pub(crate) lines: Vec<Line>,
    pub(crate) oids: OidMap,
    /// The number of new SCRAM exchanges.
    pub(crate) scram: usize,
    /// The number of `CancelRequest` messages that got the key of the server.
    pub(crate) cancels: usize,
    /// Timeouts and errors, one line each.
    pub(crate) notes: Vec<String>,
}

/// One frontend line of the trace, with the backend messages of its connection that come before it.
#[derive(Debug)]
struct Step {
    conn: u32,
    /// The frontend event, or None for the backend messages after the last frontend line of the connection.
    event: Option<Event>,
    line: Option<Line>,
    pending: Vec<Msg>,
    pending_end: bool,
}

/// Cuts a trace into steps, in the order of the trace.
fn steps(trace: &Trace) -> Vec<Step> {
    let mut out = Vec::new();
    let mut pending: BTreeMap<u32, (Vec<Msg>, bool)> = BTreeMap::new();
    for line in &trace.lines {
        let p = pending.entry(line.conn).or_default();
        match (line.dir, &line.event) {
            (Dir::B, Event::Msg(m)) => p.0.push(m.clone()),
            (Dir::B, Event::End) => p.1 = true,
            (Dir::F, event) => {
                let (msgs, end) = std::mem::take(p);
                out.push(Step {
                    conn: line.conn,
                    event: Some(event.clone()),
                    line: Some(line.clone()),
                    pending: msgs,
                    pending_end: end,
                });
            }
        }
    }
    for (conn, (msgs, end)) in pending {
        if !msgs.is_empty() || end {
            out.push(Step { conn, event: None, line: None, pending: msgs, pending_end: end });
        }
    }
    out
}

#[derive(Debug, Default)]
struct Session {
    stream: Option<TcpStream>,
    /// The messages of the server that replay has not written out yet.
    got: VecDeque<Msg>,
    eof: bool,
    /// The time of the last message to or from the server.
    last: Option<Instant>,
    auth: Auth,
    server_first: Vec<u8>,
    /// The column types of the last `RowDescription` of the server.
    types: Vec<i64>,
    end_written: bool,
    trace_key: Option<(i64, Vec<u8>)>,
    key: Option<(i64, Vec<u8>)>,
}

impl Session {
    fn quiet(&self, now: Instant) -> Duration {
        self.last.map_or(Duration::MAX, |t| now.duration_since(t))
    }

    /// How many received messages answer `step`, or None when replay must wait for more.
    fn answer(&self, step: &Step, now: Instant) -> Option<usize> {
        let done = self.eof || self.stream.is_none();
        if step.pending_end {
            return done.then_some(self.got.len());
        }
        let terminators = step.pending.iter().filter(|m| is_terminator(m)).count();
        let tail =
            step.pending.len() - step.pending.iter().rposition(is_terminator).map_or(0, |i| i + 1);
        let mut n = 0;
        let mut seen = 0;
        while seen < terminators {
            let Some(m) = self.got.get(n) else { return done.then_some(n) };
            seen += usize::from(is_terminator(m));
            n += 1;
        }
        if self.got.len() - n >= tail {
            return Some(n + tail);
        }
        (done || self.quiet(now) >= IDLE).then_some(self.got.len())
    }
}

struct Runner<'a> {
    target: &'a Target,
    timeout: Duration,
    sessions: BTreeMap<u32, Session>,
    tx: Sender<(u32, Option<Msg>)>,
    rx: Receiver<(u32, Option<Msg>)>,
    out: Replayed,
}

/// Replays a trace on one server. `timeout` is the longest wait for one answer.
///
/// Each connection keeps the order of its own lines. Across connections, replay sends the lines in the order of the trace. When the connection of the next line waits for its server and the server is quiet for a moment, replay sends the next lines of the other connections. A lock can make one connection wait for another, and the order in which the server got the messages of two connections can differ from the order in the trace, so a strict order can stop the replay.
pub(crate) fn replay(trace: &Trace, target: &Target, timeout: Duration) -> Replayed {
    let (tx, rx) = mpsc::channel();
    let mut r =
        Runner { target, timeout, sessions: BTreeMap::new(), tx, rx, out: Replayed::default() };
    for line in &trace.lines {
        if let (Dir::B, Event::Msg(m)) = (line.dir, &line.event)
            && m.name() == "BackendKeyData"
        {
            r.sessions.entry(line.conn).or_default().trace_key = key_of(m);
        }
    }
    let steps = steps(trace);
    let mut queues: BTreeMap<u32, VecDeque<usize>> = BTreeMap::new();
    for (i, s) in steps.iter().enumerate() {
        queues.entry(s.conn).or_default().push_back(i);
        r.sessions.entry(s.conn).or_default();
    }
    // The first step of the trace that waits, and since when it waits.
    let mut waiting: Option<(usize, Instant)> = None;
    loop {
        r.drain(Duration::ZERO);
        let mut heads: Vec<usize> = queues.values().filter_map(|q| q.front().copied()).collect();
        if heads.is_empty() {
            break;
        }
        heads.sort_unstable();
        let now = Instant::now();
        let first = &steps[heads[0]];
        let quiet = r.sessions.get(&first.conn).map_or(Duration::MAX, |s| s.quiet(now));
        let mut chosen = r.ready(first, now).map(|n| (heads[0], n));
        if chosen.is_none() && quiet >= IDLE {
            chosen = heads[1..].iter().find_map(|&i| r.ready(&steps[i], now).map(|n| (i, n)));
        }
        if waiting.is_none_or(|(i, _)| i != heads[0]) {
            waiting = Some((heads[0], now));
        }
        let since = waiting.map_or(now, |(_, t)| t);
        if chosen.is_none() && quiet >= r.timeout && now.duration_since(since) >= r.timeout {
            r.note(format!("connection {}: no answer in {} s", first.conn, r.timeout.as_secs()));
            chosen = Some((heads[0], r.sessions[&first.conn].got.len()));
        }
        match chosen {
            Some((i, n)) => {
                queues.get_mut(&steps[i].conn).and_then(VecDeque::pop_front);
                r.run(&steps[i], n);
            }
            None => r.drain(TICK),
        }
    }
    // Take what the servers still send on the connections that the trace left open, then close them.
    loop {
        r.drain(TICK);
        let now = Instant::now();
        if r.sessions.values().filter(|s| !s.eof).all(|s| s.quiet(now) >= IDLE) {
            break;
        }
    }
    let conns: Vec<u32> = r.sessions.keys().copied().collect();
    for conn in conns {
        let n = r.sessions[&conn].got.len();
        r.take(conn, &[], n);
        if let Some(stream) = &r.sessions[&conn].stream {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
    r.out
}

/// How long replay waits for a message before it looks at the connections again.
const TICK: Duration = Duration::from_millis(10);

impl Runner<'_> {
    fn note(&mut self, text: String) {
        self.out.notes.push(text);
    }

    /// How many received messages answer `step`, or None when replay must wait. A `CancelRequest` also waits until the server of the connection that it cancels is quiet, so the statement runs when the cancel comes, as in the trace.
    fn ready(&self, step: &Step, now: Instant) -> Option<usize> {
        let n = self.sessions.get(&step.conn)?.answer(step, now)?;
        if let Some(Event::Msg(m)) = &step.event
            && m.name() == "CancelRequest"
        {
            let want = key_of(m);
            let target =
                self.sessions.values().find(|s| s.trace_key.is_some() && s.trace_key == want);
            if target.is_some_and(|s| !s.eof && s.stream.is_some() && s.quiet(now) < IDLE) {
                return None;
            }
        }
        Some(n)
    }

    /// Moves the messages that the servers sent into their sessions. Waits up to `wait` for the first one.
    fn drain(&mut self, wait: Duration) {
        let mut next =
            if wait.is_zero() { self.rx.try_recv().ok() } else { self.rx.recv_timeout(wait).ok() };
        while let Some((conn, msg)) = next {
            let s = self.sessions.entry(conn).or_default();
            match msg {
                Some(m) => {
                    s.got.push_back(m);
                    s.last = Some(Instant::now());
                }
                None => s.eof = true,
            }
            next = self.rx.try_recv().ok();
        }
    }

    /// Writes out the answer to one step, then sends its frontend line.
    fn run(&mut self, step: &Step, n: usize) {
        self.take(step.conn, &step.pending, n);
        let (Some(event), Some(line)) = (&step.event, &step.line) else { return };
        self.out.lines.push(line.clone());
        match event {
            Event::Msg(m) => self.send(step.conn, m),
            Event::End => {
                if let Some(stream) = &self.sessions.entry(step.conn).or_default().stream {
                    let _ = stream.shutdown(Shutdown::Write);
                }
            }
        }
    }

    /// Takes the first `n` messages that a server sent: follows the authentication, the keys and the row types, learns the OIDs from the messages of the trace at the same place, and writes the messages out with the OIDs of the trace.
    fn take(&mut self, conn: u32, pending: &[Msg], n: usize) {
        for i in 0..n {
            let s = self.sessions.entry(conn).or_default();
            let Some(msg) = s.got.pop_front() else { break };
            let checked = s.auth.see(&msg);
            match msg.name() {
                "AuthenticationSASLContinue" => {
                    s.server_first = msg.vals[0].bytes().unwrap_or_default().to_vec();
                }
                "BackendKeyData" => s.key = key_of(&msg),
                "RowDescription" => s.types = row_types(&msg),
                _ => {}
            }
            let types = s.types.clone();
            if let Some(t) = pending.get(i) {
                learn(&mut self.out.oids, t, &msg, &types);
            }
            let mut shown = msg;
            map_backend(&mut self.out.oids, &mut shown, &types);
            self.out.lines.push(Line { conn, dir: Dir::B, event: Event::Msg(shown) });
            if let Err(e) = checked {
                self.note(format!("connection {conn}: {e}"));
            }
        }
        let s = self.sessions.entry(conn).or_default();
        if s.eof && s.got.is_empty() && !s.end_written {
            s.end_written = true;
            self.out.lines.push(Line { conn, dir: Dir::B, event: Event::End });
        }
    }

    fn open(&mut self, conn: u32) -> Result<(), String> {
        let stream = TcpStream::connect_timeout(&self.target.addr, Duration::from_secs(10))
            .map_err(|e| format!("connect to {}: {e}", self.target.addr))?;
        stream.set_nodelay(true).map_err(|e| e.to_string())?;
        let mut reader = stream.try_clone().map_err(|e| e.to_string())?;
        let tx = self.tx.clone();
        thread::spawn(move || {
            while let Ok(Some(f)) = frame::read_typed(&mut reader) {
                if tx.send((conn, Some(Msg::decode(Dir::B, &f, "")))).is_err() {
                    return;
                }
            }
            let _ = tx.send((conn, None));
        });
        let s = self.sessions.entry(conn).or_default();
        s.stream = Some(stream);
        Ok(())
    }

    fn send(&mut self, conn: u32, original: &Msg) {
        if self.sessions.get(&conn).is_none_or(|s| s.stream.is_none())
            && let Err(e) = self.open(conn)
        {
            self.note(format!("connection {conn}: {e}"));
            return;
        }
        let msg = match self.rewrite(conn, original) {
            Ok(m) => m,
            Err(e) => {
                self.note(format!("connection {conn}: {e}"));
                original.clone()
            }
        };
        let s = self.sessions.entry(conn).or_default();
        s.last = Some(Instant::now());
        let Some(stream) = s.stream.as_mut() else { return };
        if let Err(e) = msg.encode().write_to(stream) {
            let text = format!("connection {conn}: send {}: {e}", msg.name());
            self.note(text);
        }
    }

    fn rewrite(&mut self, conn: u32, original: &Msg) -> Result<Msg, String> {
        let password = self.target.password.clone();
        let s = self.sessions.entry(conn).or_default();
        let mut msg = original.clone();
        match msg.name() {
            "SASLInitialResponse" => {
                self.out.scram += 1;
                return Ok(s.auth.sasl_initial(&password));
            }
            "SASLResponse" => return s.auth.sasl_response(&s.server_first.clone()),
            "PasswordMessage" if s.auth.last == Some("AuthenticationCleartextPassword") => {
                return Ok(Msg::new(Dir::F, "PasswordMessage", vec![str_val(&password)]));
            }
            "CancelRequest" => {
                let want = key_of(&msg);
                let key =
                    self.sessions.values().find(|s| s.trace_key.is_some() && s.trace_key == want);
                let Some((pid, key)) = key.and_then(|s| s.key.clone()) else {
                    return Err("the CancelRequest names no connection of the trace".into());
                };
                self.out.cancels += 1;
                msg.vals = vec![Val::Int(pid), Val::Str(key)];
            }
            "StartupMessage" => add_params(&mut msg, &self.target.params),
            _ => map_frontend(&mut self.out.oids, &mut msg),
        }
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gaps_between_user_oids_are_compared() {
        let mut map = OidMap::default();
        for (t, s) in [(16_384, 20_000), (16_387, 20_003), (16_390, 20_010)] {
            map.learn(t, s);
        }
        assert_eq!(map.gaps(), (2, 1));
    }

    #[test]
    fn the_map_rewrites_whole_numbers_only() {
        let mut m = OidMap::default();
        m.learn(16_400, 16_500);
        m.learn(100, 200);
        m.learn(16_401, 16_500);
        assert_eq!(m.len(), 1);
        let sql = b"SELECT 16400, '16400', x16400, 16400.5, 164001, 1.16400, c.oid = 16400";
        let out = m.map_sql(sql, false);
        assert_eq!(out, b"SELECT 16500, '16500', x16400, 16400.5, 164001, 1.16400, c.oid = 16500");
        assert_eq!(m.applied, 3);
        assert_eq!(m.map_sql(&out, true), sql);
    }

    #[test]
    fn row_descriptions_and_oid_columns_teach_the_map() {
        let field = |table: i64, ty: i64| {
            Val::Tuple(vec![
                str_val("c"),
                Val::Int(table),
                Val::Int(1),
                Val::Int(ty),
                Val::Int(4),
                Val::Int(-1),
                Val::Int(0),
            ])
        };
        let rd = |table, ty| {
            Msg::new(
                Dir::B,
                "RowDescription",
                vec![Val::List(vec![field(table, 26), field(0, ty)])],
            )
        };
        let dr = |a: &str| {
            Msg::new(Dir::B, "DataRow", vec![Val::List(vec![str_val(a), str_val("16390")])])
        };
        let mut m = OidMap::default();
        learn(&mut m, &rd(16_384, 16_386), &rd(17_000, 17_002), &[]);
        learn(&mut m, &dr("16388"), &dr("17004"), &[26, 23]);
        assert_eq!(m.len(), 3);
        let mut got = dr("17004");
        map_backend(&mut m, &mut got, &[26, 23]);
        assert_eq!(got, dr("16388"));
        let mut q =
            Msg::new(Dir::F, "Query", vec![str_val("SELECT * FROM pg_class WHERE oid = 16384")]);
        map_frontend(&mut m, &mut q);
        assert_eq!(q.vals[0], str_val("SELECT * FROM pg_class WHERE oid = 17000"));
        let mut b = Msg::new(
            Dir::F,
            "Bind",
            vec![
                str_val(""),
                str_val(""),
                Val::List(vec![]),
                Val::List(vec![str_val("16386"), Val::Null]),
                Val::List(vec![]),
            ],
        );
        map_frontend(&mut m, &mut b);
        assert_eq!(b.vals[3].list()[0], str_val("17002"));
    }

    #[test]
    fn terminators() {
        assert!(is_terminator(&Msg::new(Dir::B, "ReadyForQuery", vec![Val::Byte(b'I')])));
        assert!(is_terminator(&Msg::new(Dir::B, "AuthenticationSASLContinue", vec![str_val("")])));
        assert!(!is_terminator(&Msg::new(Dir::B, "AuthenticationSASLFinal", vec![str_val("")])));
        assert!(!is_terminator(&Msg::new(Dir::B, "CommandComplete", vec![str_val("SELECT 1")])));
    }

    fn line(conn: u32, dir: Dir, name: &str, vals: Vec<Val>) -> Line {
        Line { conn, dir, event: Event::Msg(Msg::new(dir, name, vals)) }
    }

    #[test]
    fn steps_carry_the_backend_messages_of_their_connection() {
        let rfq = || vec![Val::Byte(b'I')];
        let trace = Trace {
            header: Vec::new(),
            lines: vec![
                line(1, Dir::F, "Query", vec![str_val("update t set a = 1")]),
                line(2, Dir::F, "Query", vec![str_val("update t set a = 2")]),
                line(1, Dir::B, "CommandComplete", vec![str_val("UPDATE 1")]),
                line(1, Dir::B, "ReadyForQuery", rfq()),
                line(1, Dir::F, "Query", vec![str_val("commit")]),
                line(2, Dir::B, "ReadyForQuery", rfq()),
                line(1, Dir::B, "ReadyForQuery", rfq()),
                Line { conn: 1, dir: Dir::B, event: Event::End },
            ],
        };
        let steps = steps(&trace);
        let shape: Vec<(u32, bool, usize, bool)> = steps
            .iter()
            .map(|s| (s.conn, s.event.is_some(), s.pending.len(), s.pending_end))
            .collect();
        assert_eq!(
            shape,
            vec![
                (1, true, 0, false),
                (2, true, 0, false),
                (1, true, 2, false),
                (1, false, 1, true),
                (2, false, 1, false)
            ]
        );
    }

    #[test]
    fn a_step_waits_for_its_terminators_and_its_tail() {
        let now = Instant::now();
        let rfq = Msg::new(Dir::B, "ReadyForQuery", vec![Val::Byte(b'I')]);
        let done = Msg::new(Dir::B, "CommandComplete", vec![str_val("SELECT 1")]);
        let step = Step {
            conn: 1,
            event: None,
            line: None,
            pending: vec![done.clone(), rfq.clone(), done.clone()],
            pending_end: false,
        };
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let mut s = Session {
            stream: Some(TcpStream::connect(listener.local_addr().unwrap()).unwrap()),
            last: Some(now),
            ..Session::default()
        };
        assert_eq!(s.answer(&step, now), None);
        s.got.extend([done.clone(), rfq.clone()]);
        assert_eq!(s.answer(&step, now), None, "the tail is not there yet");
        assert_eq!(s.answer(&step, now + IDLE), Some(2), "a quiet server ends the tail");
        s.got.extend([done.clone(), rfq.clone()]);
        assert_eq!(s.answer(&step, now), Some(3), "the next answer stays for the next step");
        s.stream = None;
        s.got.clear();
        assert_eq!(s.answer(&step, now), Some(0), "a closed connection does not wait");
    }
}
