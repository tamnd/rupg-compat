//! The text trace format: one message on each line (spec/21 section 21.3.5).
//!
//! A line is `<connection> <F|B> <message name> <fields>`. `F` is a message from the frontend and `B` a message from the backend. The fields follow the schema of the message in `message.rs`:
//!
//! - an integer in decimal, and a protocol version as `3.0` or `3.2`;
//! - a byte as its letter or digit, else as `0xHH`;
//! - a string in double quotes, with `\\`, `\"`, `\n`, `\r`, `\t` and `\xHH` escapes for bytes that are not printable or not UTF-8;
//! - `NULL` for a null value of a `DataRow`, `Bind` or `FunctionCall`;
//! - an array in `[ ]`, and a group of an array with more than one field in `( )`;
//! - the fields of `ErrorResponse` and `NoticeResponse`, the parameters of `StartupMessage` and the SASL mechanisms as a flat list to the end of the line.
//!
//! `<connection> F End` and `<connection> B End` mark the end of a stream. A line that starts with `#` is a comment. For example:
//!
//! ```text
//! 1 F StartupMessage 3.0 "user" "postgres" "database" "postgres"
//! 1 B ReadyForQuery I
//! 1 F Query "SELECT 1"
//! 1 B RowDescription [("?column?" 0 0 23 4 -1 0)]
//! 1 B DataRow ["1"]
//! 1 B CommandComplete "SELECT 1"
//! 1 B ErrorResponse S "ERROR" V "ERROR" C "42601" M "syntax error at or near \"x\""
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{self, Write};

use crate::message::{Dir, Kind, Msg, Val, spec_by_name};

/// One event of a trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Msg(Msg),
    /// The end of the stream in this direction.
    End,
}

/// One line of a trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) conn: u32,
    pub(crate) dir: Dir,
    pub(crate) event: Event,
}

impl Line {
    pub(crate) fn msg(&self) -> Option<&Msg> {
        match &self.event {
            Event::Msg(m) => Some(m),
            Event::End => None,
        }
    }
}

/// A whole trace: the comments at the top, then the lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Trace {
    pub(crate) header: Vec<String>,
    pub(crate) lines: Vec<Line>,
}

impl Trace {
    pub(crate) fn parse(text: &str) -> Result<Trace, String> {
        let mut trace = Trace::default();
        for (i, raw) in text.lines().enumerate() {
            let l = raw.trim_end_matches('\r');
            if l.trim().is_empty() {
                continue;
            }
            if let Some(c) = l.strip_prefix('#') {
                if trace.lines.is_empty() {
                    trace.header.push(c.trim().to_string());
                }
                continue;
            }
            trace.lines.push(parse_line(l).map_err(|e| format!("line {}: {e}", i + 1))?);
        }
        Ok(trace)
    }

    pub(crate) fn read(path: &std::path::Path) -> Result<Trace, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Trace::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    #[cfg(test)]
    pub(crate) fn to_text(&self) -> String {
        let mut out = String::new();
        for h in &self.header {
            let _ = writeln!(out, "# {h}");
        }
        for l in &self.lines {
            out.push_str(&format_line(l));
            out.push('\n');
        }
        out
    }

    /// The value of a `# key: value` header line.
    pub(crate) fn header_value(&self, key: &str) -> Option<&str> {
        self.header.iter().find_map(|h| h.strip_prefix(key)?.strip_prefix(':').map(str::trim))
    }
}

/// Writes one line, without the line end.
pub(crate) fn format_line(l: &Line) -> String {
    let dir = match l.dir {
        Dir::F => 'F',
        Dir::B => 'B',
    };
    match &l.event {
        Event::End => format!("{} {dir} End", l.conn),
        Event::Msg(m) => format!("{} {dir} {}", l.conn, format_msg(m)),
    }
}

/// Writes a message as `<name> <fields>`.
pub(crate) fn format_msg(m: &Msg) -> String {
    let mut out = m.name().to_string();
    for (kind, val) in m.spec.fields.iter().zip(&m.vals) {
        if is_flat(*kind) && val.list().is_empty() {
            continue;
        }
        out.push(' ');
        write_val(&mut out, *kind, val);
    }
    out
}

fn is_flat(kind: Kind) -> bool {
    matches!(kind, Kind::Fields | Kind::Pairs | Kind::StrList)
}

fn write_val(out: &mut String, kind: Kind, val: &Val) {
    match (kind, val) {
        (_, Val::Null) => out.push_str("NULL"),
        (Kind::Version, Val::Int(v)) => {
            let _ = write!(out, "{}.{}", v >> 16, v & 0xffff);
        }
        (_, Val::Int(i)) => {
            let _ = write!(out, "{i}");
        }
        (_, Val::Byte(b)) => write_byte(out, *b),
        (_, Val::Str(s)) => write_str(out, s),
        (Kind::Array16(group) | Kind::Array32(group), Val::List(items)) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                if group.len() == 1 {
                    write_val(out, group[0], item);
                } else {
                    out.push('(');
                    for (j, (k, v)) in group.iter().zip(item.list()).enumerate() {
                        if j > 0 {
                            out.push(' ');
                        }
                        write_val(out, *k, v);
                    }
                    out.push(')');
                }
            }
            out.push(']');
        }
        (_, Val::List(items) | Val::Tuple(items)) => {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                write_val(out, Kind::Str, item);
            }
        }
    }
}

fn write_byte(out: &mut String, b: u8) {
    if b.is_ascii_alphanumeric() {
        out.push(char::from(b));
    } else {
        let _ = write!(out, "0x{b:02x}");
    }
}

/// Writes bytes as a quoted string.
pub(crate) fn write_str(out: &mut String, s: &[u8]) {
    out.push('"');
    for chunk in s.utf8_chunks() {
        for c in chunk.valid().chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    let mut buf = [0u8; 4];
                    for b in c.encode_utf8(&mut buf).bytes() {
                        let _ = write!(out, "\\x{b:02x}");
                    }
                }
                c => out.push(c),
            }
        }
        for b in chunk.invalid() {
            let _ = write!(out, "\\x{b:02x}");
        }
    }
    out.push('"');
}

#[derive(Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Str(Vec<u8>),
    Open(char),
    Close(char),
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let mut toks = Vec::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            ' ' | '\t' => {}
            '[' | '(' => toks.push(Tok::Open(c)),
            ']' | ')' => toks.push(Tok::Close(c)),
            '"' => {
                let mut buf = Vec::new();
                loop {
                    let Some((_, c)) = chars.next() else {
                        return Err("unterminated string".into());
                    };
                    match c {
                        '"' => break,
                        '\\' => match chars.next().map(|(_, c)| c) {
                            Some('n') => buf.push(b'\n'),
                            Some('r') => buf.push(b'\r'),
                            Some('t') => buf.push(b'\t'),
                            Some('"') => buf.push(b'"'),
                            Some('\\') => buf.push(b'\\'),
                            Some('x') => {
                                let hex: String =
                                    (0..2).filter_map(|_| chars.next().map(|(_, c)| c)).collect();
                                buf.push(
                                    u8::from_str_radix(&hex, 16)
                                        .map_err(|_| format!("bad escape \\x{hex}"))?,
                                );
                            }
                            other => return Err(format!("bad escape {other:?}")),
                        },
                        c => {
                            let mut b = [0u8; 4];
                            buf.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
                        }
                    }
                }
                toks.push(Tok::Str(buf));
            }
            _ => {
                let mut end = i + c.len_utf8();
                while let Some(&(j, d)) = chars.peek() {
                    if d == ' ' || d == '\t' || "[]()\"".contains(d) {
                        break;
                    }
                    end = j + d.len_utf8();
                    chars.next();
                }
                toks.push(Tok::Word(s[i..end].to_string()));
            }
        }
    }
    Ok(toks)
}

/// Reads one line.
pub(crate) fn parse_line(l: &str) -> Result<Line, String> {
    let mut toks = tokenize(l)?.into_iter();
    let conn = match toks.next() {
        Some(Tok::Word(w)) => w.parse().map_err(|_| format!("bad connection number {w}"))?,
        _ => return Err("expected a connection number".into()),
    };
    let dir = match toks.next() {
        Some(Tok::Word(w)) if w == "F" => Dir::F,
        Some(Tok::Word(w)) if w == "B" => Dir::B,
        _ => return Err("expected F or B".into()),
    };
    let name = match toks.next() {
        Some(Tok::Word(w)) => w,
        _ => return Err("expected a message name".into()),
    };
    let rest: Vec<Tok> = toks.collect();
    if name == "End" {
        return Ok(Line { conn, dir, event: Event::End });
    }
    let spec = spec_by_name(dir, &name).ok_or_else(|| format!("unknown message {name}"))?;
    let mut p = Toks { toks: rest, pos: 0 };
    let mut vals = Vec::new();
    for kind in spec.fields {
        vals.push(p.val(*kind)?);
    }
    if p.pos != p.toks.len() {
        return Err(format!("too many fields for {name}"));
    }
    Ok(Line { conn, dir, event: Event::Msg(Msg { spec, vals }) })
}

struct Toks {
    toks: Vec<Tok>,
    pos: usize,
}

impl Toks {
    fn next(&mut self) -> Option<&Tok> {
        let t = self.toks.get(self.pos);
        self.pos += 1;
        t
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn val(&mut self, kind: Kind) -> Result<Val, String> {
        if is_flat(kind) {
            let mut items = Vec::new();
            while self.peek().is_some() {
                let k = if kind == Kind::Fields && items.len() % 2 == 0 {
                    Kind::Byte
                } else {
                    Kind::Str
                };
                items.push(self.val(k)?);
            }
            return Ok(Val::List(items));
        }
        let tok = self.next().ok_or_else(|| format!("missing {kind:?} field"))?;
        Ok(match (kind, tok) {
            (Kind::Bytes32, Tok::Word(w)) if w == "NULL" => Val::Null,
            (Kind::Str | Kind::Rest | Kind::Bytes32, Tok::Str(s)) => Val::Str(s.clone()),
            (Kind::Byte, Tok::Word(w)) => Val::Byte(parse_byte(w)?),
            (Kind::Version, Tok::Word(w)) => {
                let (major, minor) = w.split_once('.').ok_or_else(|| format!("bad version {w}"))?;
                let major: i64 = major.parse().map_err(|_| format!("bad version {w}"))?;
                let minor: i64 = minor.parse().map_err(|_| format!("bad version {w}"))?;
                Val::Int(major << 16 | minor)
            }
            (Kind::I8 | Kind::I16 | Kind::I32, Tok::Word(w)) => {
                Val::Int(w.parse().map_err(|_| format!("bad integer {w}"))?)
            }
            (Kind::Array16(group) | Kind::Array32(group), Tok::Open('[')) => {
                let mut items = Vec::new();
                loop {
                    match self.peek() {
                        Some(Tok::Close(']')) => {
                            self.pos += 1;
                            break;
                        }
                        None => return Err("unterminated array".into()),
                        _ if group.len() == 1 => items.push(self.val(group[0])?),
                        _ => {
                            if self.next() != Some(&Tok::Open('(')) {
                                return Err("expected ( in an array".into());
                            }
                            let mut vals = Vec::new();
                            for k in group.iter() {
                                vals.push(self.val(*k)?);
                            }
                            if self.next() != Some(&Tok::Close(')')) {
                                return Err("expected )".into());
                            }
                            items.push(Val::Tuple(vals));
                        }
                    }
                }
                Val::List(items)
            }
            (kind, tok) => return Err(format!("expected {kind:?}, found {tok:?}")),
        })
    }
}

fn parse_byte(w: &str) -> Result<u8, String> {
    if let Some(hex) = w.strip_prefix("0x") {
        return u8::from_str_radix(hex, 16).map_err(|_| format!("bad byte {w}"));
    }
    match w.as_bytes() {
        [b] => Ok(*b),
        _ => Err(format!("bad byte {w}")),
    }
}

/// Writes lines to a file as they come.
#[derive(Debug)]
pub(crate) struct Writer<W: Write> {
    out: W,
}

impl<W: Write> Writer<W> {
    pub(crate) fn new(mut out: W, header: &[String]) -> io::Result<Writer<W>> {
        for h in header {
            writeln!(out, "# {h}")?;
        }
        Ok(Writer { out })
    }

    pub(crate) fn line(&mut self, l: &Line) -> io::Result<()> {
        writeln!(self.out, "{}", format_line(l))
    }

    pub(crate) fn comment(&mut self, text: &str) -> io::Result<()> {
        writeln!(self.out, "# {}", text.replace('\n', " "))
    }

    pub(crate) fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

/// Checks that each message of a trace comes back the same after a trip through the wire format, and counts the messages by name.
pub(crate) fn check(trace: &Trace) -> Result<BTreeMap<&'static str, usize>, String> {
    let mut counts = BTreeMap::new();
    for l in &trace.lines {
        let Some(msg) = l.msg() else { continue };
        let back = Msg::decode(msg.dir(), &msg.encode(), msg.name());
        if &back != msg {
            return Err(format!(
                "{} is not the same after the wire format: {}",
                format_line(l),
                format_msg(&back)
            ));
        }
        *counts.entry(msg.name()).or_insert(0) += 1;
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::tests::samples;

    #[test]
    fn every_message_round_trips_through_text() {
        for (i, msg) in samples().into_iter().enumerate() {
            let line = Line { conn: i as u32 + 1, dir: msg.dir(), event: Event::Msg(msg) };
            let text = format_line(&line);
            assert!(!text.contains('\n'), "{text}");
            assert_eq!(parse_line(&text).as_ref(), Ok(&line), "{text}");
        }
    }

    #[test]
    fn the_example_of_the_module_doc_parses() {
        let text = "# client: psql\n1 F StartupMessage 3.0 \"user\" \"postgres\" \"database\" \"postgres\"\n1 B ReadyForQuery I\n1 F Query \"SELECT 1\"\n1 B RowDescription [(\"?column?\" 0 0 23 4 -1 0)]\n1 B DataRow [\"1\"]\n1 B CommandComplete \"SELECT 1\"\n1 B ErrorResponse S \"ERROR\" V \"ERROR\" C \"42601\" M \"syntax error at or near \\\"x\\\"\"\n1 F End\n";
        let trace = Trace::parse(text).unwrap();
        assert_eq!(trace.header_value("client"), Some("psql"));
        assert_eq!(trace.lines.len(), 8);
        assert_eq!(trace.to_text(), text);
        let row = trace.lines[3].msg().unwrap();
        assert_eq!(row.vals[0].list()[0].list()[3], Val::Int(23));
        let err = trace.lines[6].msg().unwrap();
        assert_eq!(err.vals[0].list()[7], Val::Str(b"syntax error at or near \"x\"".to_vec()));
    }

    #[test]
    fn strings_keep_every_byte() {
        let bytes: Vec<u8> = (0..=255).collect();
        let mut s = String::new();
        write_str(&mut s, &bytes);
        assert!(!s.contains('\n'));
        let toks = tokenize(&s).unwrap();
        assert_eq!(toks, vec![Tok::Str(bytes)]);
        let mut s = String::new();
        write_str(&mut s, "é ü 日本".as_bytes());
        assert_eq!(s, "\"é ü 日本\"");
    }

    #[test]
    fn errors() {
        for bad in [
            "x F Query \"a\"",
            "1 X Query \"a\"",
            "1 F Nope",
            "1 F Query",
            "1 F Query \"a\" \"b\"",
            "1 B DataRow [\"a\"",
            "1 F Query \"a",
        ] {
            assert!(parse_line(bad).is_err(), "{bad}");
        }
    }
}
