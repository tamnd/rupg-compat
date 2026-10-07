//! The messages of the PostgreSQL v3 protocol, with a schema for each one.
//!
//! A schema is a list of field kinds. One decoder and one encoder walk the schema, so a message type is one line in `SPECS`. The trace format of `trace.rs` writes the same fields as text. A frame that matches no schema becomes a `Message` with its tag and its raw body, so nothing is lost.

use crate::frame::{CANCEL_REQUEST, Frame, GSSENC_REQUEST, SSL_REQUEST};

/// The direction of a message: from the frontend (the client) or from the backend (the server).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Dir {
    F,
    B,
}

/// The kind of one field in a schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `Byte1`, written as a character.
    Byte,
    /// `Int8`, `Int16` and `Int32`, written in decimal.
    I8,
    I16,
    I32,
    /// A protocol version in an `Int32`, written as `3.0`.
    Version,
    /// A string that ends with a zero byte.
    Str,
    /// The rest of the body.
    Rest,
    /// An `Int32` length and that many bytes, or -1 for NULL.
    Bytes32,
    /// An `Int16` count and that many groups.
    Array16(&'static [Kind]),
    /// An `Int32` count and that many groups.
    Array32(&'static [Kind]),
    /// The fields of `ErrorResponse` and `NoticeResponse`: a code byte and a string, until a zero byte.
    Fields,
    /// The parameters of `StartupMessage`: name and value strings, until an empty name.
    Pairs,
    /// The mechanism list of `AuthenticationSASL`: strings, until an empty string.
    StrList,
}

/// One field value.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Val {
    Int(i64),
    Byte(u8),
    Str(Vec<u8>),
    Null,
    /// An array, or the flat list of `Fields`, `Pairs` and `StrList`.
    List(Vec<Val>),
    /// One group of an array with more than one field.
    Tuple(Vec<Val>),
}

impl Val {
    pub(crate) fn int(&self) -> Option<i64> {
        match self {
            Val::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub(crate) fn bytes(&self) -> Option<&[u8]> {
        match self {
            Val::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn list(&self) -> &[Val] {
        match self {
            Val::List(v) | Val::Tuple(v) => v,
            _ => &[],
        }
    }
}

/// How a message is told apart on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    /// A typed message with this tag.
    Tag(u8),
    /// An `Authentication` message (tag `R`) with this subtype.
    Auth(i32),
    /// A frontend password message (tag `p`). The state of the exchange says which one it is.
    Password,
    /// An untyped frontend packet with this request code.
    Code(i32),
    /// The `StartupMessage`, whose first field is the protocol version.
    Startup,
    /// The one-byte answer to an encryption request.
    Byte,
    /// A frame that no other schema matches.
    Unknown,
}

/// The schema of one message type.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Spec {
    pub(crate) name: &'static str,
    pub(crate) dir: Dir,
    pub(crate) key: Key,
    pub(crate) fields: &'static [Kind],
}

use Kind::*;

const ROW_FIELD: &[Kind] = &[Str, I32, I16, I32, I16, I32, I16];

macro_rules! spec {
    ($name:literal, $dir:ident, $key:expr, [$($f:expr),*]) => {
        Spec { name: $name, dir: Dir::$dir, key: $key, fields: &[$($f),*] }
    };
}

/// Every message type of protocol 3.0 and 3.2.
pub(crate) static SPECS: &[Spec] = &[
    // Untyped frontend packets.
    spec!("StartupMessage", F, Key::Startup, [Version, Pairs]),
    spec!("SSLRequest", F, Key::Code(SSL_REQUEST), []),
    spec!("GSSENCRequest", F, Key::Code(GSSENC_REQUEST), []),
    spec!("CancelRequest", F, Key::Code(CANCEL_REQUEST), [I32, Rest]),
    // Typed frontend messages.
    spec!(
        "Bind",
        F,
        Key::Tag(b'B'),
        [Str, Str, Array16(&[I16]), Array16(&[Bytes32]), Array16(&[I16])]
    ),
    spec!("Close", F, Key::Tag(b'C'), [Byte, Str]),
    spec!("CopyData", F, Key::Tag(b'd'), [Rest]),
    spec!("CopyDone", F, Key::Tag(b'c'), []),
    spec!("CopyFail", F, Key::Tag(b'f'), [Str]),
    spec!("Describe", F, Key::Tag(b'D'), [Byte, Str]),
    spec!("Execute", F, Key::Tag(b'E'), [Str, I32]),
    spec!("Flush", F, Key::Tag(b'H'), []),
    spec!("FunctionCall", F, Key::Tag(b'F'), [I32, Array16(&[I16]), Array16(&[Bytes32]), I16]),
    spec!("Parse", F, Key::Tag(b'P'), [Str, Str, Array16(&[I32])]),
    spec!("Query", F, Key::Tag(b'Q'), [Str]),
    spec!("Sync", F, Key::Tag(b'S'), []),
    spec!("Terminate", F, Key::Tag(b'X'), []),
    spec!("PasswordMessage", F, Key::Password, [Str]),
    spec!("SASLInitialResponse", F, Key::Password, [Str, Bytes32]),
    spec!("SASLResponse", F, Key::Password, [Rest]),
    spec!("GSSResponse", F, Key::Password, [Rest]),
    // The answer to SSLRequest and GSSENCRequest.
    spec!("EncryptionResponse", B, Key::Byte, [Byte]),
    // Backend messages.
    spec!("AuthenticationOk", B, Key::Auth(0), []),
    spec!("AuthenticationKerberosV5", B, Key::Auth(2), []),
    spec!("AuthenticationCleartextPassword", B, Key::Auth(3), []),
    spec!("AuthenticationMD5Password", B, Key::Auth(5), [Rest]),
    spec!("AuthenticationGSS", B, Key::Auth(7), []),
    spec!("AuthenticationGSSContinue", B, Key::Auth(8), [Rest]),
    spec!("AuthenticationSSPI", B, Key::Auth(9), []),
    spec!("AuthenticationSASL", B, Key::Auth(10), [StrList]),
    spec!("AuthenticationSASLContinue", B, Key::Auth(11), [Rest]),
    spec!("AuthenticationSASLFinal", B, Key::Auth(12), [Rest]),
    spec!("BackendKeyData", B, Key::Tag(b'K'), [I32, Rest]),
    spec!("BindComplete", B, Key::Tag(b'2'), []),
    spec!("CloseComplete", B, Key::Tag(b'3'), []),
    spec!("CommandComplete", B, Key::Tag(b'C'), [Str]),
    spec!("CopyData", B, Key::Tag(b'd'), [Rest]),
    spec!("CopyDone", B, Key::Tag(b'c'), []),
    spec!("CopyInResponse", B, Key::Tag(b'G'), [I8, Array16(&[I16])]),
    spec!("CopyOutResponse", B, Key::Tag(b'H'), [I8, Array16(&[I16])]),
    spec!("CopyBothResponse", B, Key::Tag(b'W'), [I8, Array16(&[I16])]),
    spec!("DataRow", B, Key::Tag(b'D'), [Array16(&[Bytes32])]),
    spec!("EmptyQueryResponse", B, Key::Tag(b'I'), []),
    spec!("ErrorResponse", B, Key::Tag(b'E'), [Fields]),
    spec!("FunctionCallResponse", B, Key::Tag(b'V'), [Bytes32]),
    spec!("NegotiateProtocolVersion", B, Key::Tag(b'v'), [I32, Array32(&[Str])]),
    spec!("NoData", B, Key::Tag(b'n'), []),
    spec!("NoticeResponse", B, Key::Tag(b'N'), [Fields]),
    spec!("NotificationResponse", B, Key::Tag(b'A'), [I32, Str, Str]),
    spec!("ParameterDescription", B, Key::Tag(b't'), [Array16(&[I32])]),
    spec!("ParameterStatus", B, Key::Tag(b'S'), [Str, Str]),
    spec!("ParseComplete", B, Key::Tag(b'1'), []),
    spec!("PortalSuspended", B, Key::Tag(b's'), []),
    spec!("ReadyForQuery", B, Key::Tag(b'Z'), [Byte]),
    spec!("RowDescription", B, Key::Tag(b'T'), [Array16(ROW_FIELD)]),
    // A frame that matches no schema: the tag and the raw body.
    spec!("Message", F, Key::Unknown, [Byte, Rest]),
    spec!("Message", B, Key::Unknown, [Byte, Rest]),
];

/// One decoded message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Msg {
    pub(crate) spec: &'static Spec,
    pub(crate) vals: Vec<Val>,
}

pub(crate) fn spec_by_name(dir: Dir, name: &str) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.dir == dir && s.name == name)
}

fn spec_by_key(dir: Dir, key: Key) -> Option<&'static Spec> {
    SPECS.iter().find(|s| s.dir == dir && s.key == key)
}

fn unknown(dir: Dir) -> &'static Spec {
    SPECS.iter().find(|s| s.dir == dir && s.key == Key::Unknown).unwrap_or(&SPECS[0])
}

impl Msg {
    pub(crate) fn name(&self) -> &'static str {
        self.spec.name
    }

    pub(crate) fn dir(&self) -> Dir {
        self.spec.dir
    }

    /// Decodes a frame. `password` names the type of a `p` message, which depends on the state of the authentication exchange.
    pub(crate) fn decode(dir: Dir, frame: &Frame, password: &str) -> Msg {
        let (spec, body): (&'static Spec, &[u8]) = match frame {
            Frame::Byte(b) => {
                return Msg {
                    spec: spec_by_key(dir, Key::Byte).unwrap_or(unknown(dir)),
                    vals: vec![Val::Byte(*b)],
                };
            }
            Frame::Untyped(body) => match frame.code() {
                Some(code) if code >> 16 == 3 => {
                    (spec_by_key(dir, Key::Startup).unwrap_or(unknown(dir)), body)
                }
                Some(code) => match spec_by_key(dir, Key::Code(code)) {
                    Some(s) => (s, &body[4..]),
                    None => (unknown(dir), body),
                },
                None => (unknown(dir), body),
            },
            Frame::Typed(b'R', body) if dir == Dir::B && body.len() >= 4 => {
                let sub = i32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                match spec_by_key(dir, Key::Auth(sub)) {
                    Some(s) => (s, &body[4..]),
                    None => (unknown(dir), body),
                }
            }
            Frame::Typed(b'p', body) if dir == Dir::F => (
                spec_by_name(dir, password)
                    .filter(|s| s.key == Key::Password)
                    .unwrap_or(unknown(dir)),
                body,
            ),
            Frame::Typed(tag, body) => {
                (spec_by_key(dir, Key::Tag(*tag)).unwrap_or(unknown(dir)), body)
            }
        };
        if spec.key == Key::Unknown {
            return raw(dir, frame);
        }
        let mut r = Reader { b: body, pos: 0 };
        match r.fields(spec.fields) {
            Some(vals) if r.pos == body.len() => Msg { spec, vals },
            _ => raw(dir, frame),
        }
    }

    /// Encodes the message into a frame.
    pub(crate) fn encode(&self) -> Frame {
        let mut w = Vec::new();
        match self.spec.key {
            Key::Byte => {
                return Frame::Byte(match self.vals.first() {
                    Some(Val::Byte(b)) => *b,
                    _ => b'N',
                });
            }
            Key::Unknown => {
                let tag = match self.vals.first() {
                    Some(Val::Byte(b)) => *b,
                    _ => 0,
                };
                let body = self.vals.get(1).and_then(Val::bytes).unwrap_or_default().to_vec();
                return if tag == 0 { Frame::Untyped(body) } else { Frame::Typed(tag, body) };
            }
            Key::Code(code) => w.extend_from_slice(&code.to_be_bytes()),
            Key::Auth(sub) => w.extend_from_slice(&sub.to_be_bytes()),
            _ => {}
        }
        write_fields(&mut w, self.spec.fields, &self.vals);
        match self.spec.key {
            Key::Code(_) | Key::Startup => Frame::Untyped(w),
            Key::Auth(_) => Frame::Typed(b'R', w),
            Key::Password => Frame::Typed(b'p', w),
            Key::Tag(t) => Frame::Typed(t, w),
            Key::Byte | Key::Unknown => unreachable!("handled above"),
        }
    }
}

/// A frame kept as its tag and raw body. An untyped frame has the tag 0.
fn raw(dir: Dir, frame: &Frame) -> Msg {
    let (tag, body) = match frame {
        Frame::Untyped(b) => (0, b.clone()),
        Frame::Typed(t, b) => (*t, b.clone()),
        Frame::Byte(b) => (0, vec![*b]),
    };
    Msg { spec: unknown(dir), vals: vec![Val::Byte(tag), Val::Str(body)] }
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.b.len())?;
        let s = &self.b[self.pos..end];
        self.pos = end;
        Some(s)
    }

    fn int(&mut self, n: usize) -> Option<i64> {
        let s = self.take(n)?;
        Some(match n {
            1 => i64::from(s[0] as i8),
            2 => i64::from(i16::from_be_bytes([s[0], s[1]])),
            _ => i64::from(i32::from_be_bytes([s[0], s[1], s[2], s[3]])),
        })
    }

    fn cstr(&mut self) -> Option<Vec<u8>> {
        let rest = &self.b[self.pos..];
        let end = rest.iter().position(|&c| c == 0)?;
        let s = rest[..end].to_vec();
        self.pos += end + 1;
        Some(s)
    }

    fn fields(&mut self, kinds: &[Kind]) -> Option<Vec<Val>> {
        kinds.iter().map(|k| self.field(*k)).collect()
    }

    fn group(&mut self, kinds: &[Kind]) -> Option<Val> {
        let mut vals = self.fields(kinds)?;
        Some(if vals.len() == 1 { vals.remove(0) } else { Val::Tuple(vals) })
    }

    fn field(&mut self, kind: Kind) -> Option<Val> {
        Some(match kind {
            Byte => Val::Byte(self.take(1)?[0]),
            I8 => Val::Int(self.int(1)?),
            I16 => Val::Int(self.int(2)?),
            I32 | Version => Val::Int(self.int(4)?),
            Str => Val::Str(self.cstr()?),
            Rest => Val::Str(self.take(self.b.len() - self.pos)?.to_vec()),
            Bytes32 => match self.int(4)? {
                -1 => Val::Null,
                n => Val::Str(self.take(usize::try_from(n).ok()?)?.to_vec()),
            },
            Array16(group) | Array32(group) => {
                let n = self.int(if matches!(kind, Array16(_)) { 2 } else { 4 })?;
                let n = usize::try_from(n).ok()?;
                let mut items = Vec::with_capacity(n.min(4096));
                for _ in 0..n {
                    items.push(self.group(group)?);
                }
                Val::List(items)
            }
            Fields => {
                let mut items = Vec::new();
                loop {
                    let code = self.take(1)?[0];
                    if code == 0 {
                        break Val::List(items);
                    }
                    items.push(Val::Byte(code));
                    items.push(Val::Str(self.cstr()?));
                }
            }
            Pairs | StrList => {
                let mut items = Vec::new();
                loop {
                    let s = self.cstr()?;
                    if s.is_empty() {
                        break Val::List(items);
                    }
                    items.push(Val::Str(s));
                    if kind == Pairs {
                        items.push(Val::Str(self.cstr()?));
                    }
                }
            }
        })
    }
}

fn write_fields(w: &mut Vec<u8>, kinds: &[Kind], vals: &[Val]) {
    for (kind, val) in kinds.iter().zip(vals) {
        write_field(w, *kind, val);
    }
}

fn write_field(w: &mut Vec<u8>, kind: Kind, val: &Val) {
    let int = val.int().unwrap_or(0);
    match kind {
        Byte => w.push(match val {
            Val::Byte(b) => *b,
            _ => 0,
        }),
        I8 => w.push(int as u8),
        I16 => w.extend_from_slice(&(int as i16).to_be_bytes()),
        I32 | Version => w.extend_from_slice(&(int as i32).to_be_bytes()),
        Str => {
            w.extend_from_slice(val.bytes().unwrap_or_default());
            w.push(0);
        }
        Rest => w.extend_from_slice(val.bytes().unwrap_or_default()),
        Bytes32 => match val {
            Val::Str(s) => {
                w.extend_from_slice(&(s.len() as i32).to_be_bytes());
                w.extend_from_slice(s);
            }
            _ => w.extend_from_slice(&(-1i32).to_be_bytes()),
        },
        Array16(group) | Array32(group) => {
            let items = val.list();
            if matches!(kind, Array16(_)) {
                w.extend_from_slice(&(items.len() as i16).to_be_bytes());
            } else {
                w.extend_from_slice(&(items.len() as i32).to_be_bytes());
            }
            for item in items {
                if group.len() == 1 {
                    write_field(w, group[0], item);
                } else {
                    write_fields(w, group, item.list());
                }
            }
        }
        Fields => {
            for v in val.list() {
                match v {
                    Val::Byte(b) => w.push(*b),
                    other => {
                        w.extend_from_slice(other.bytes().unwrap_or_default());
                        w.push(0);
                    }
                }
            }
            w.push(0);
        }
        Pairs | StrList => {
            for v in val.list() {
                w.extend_from_slice(v.bytes().unwrap_or_default());
                w.push(0);
            }
            w.push(0);
        }
    }
}

/// Follows the authentication exchange to name the next `p` message from the frontend.
pub(crate) fn password_name(last_auth: Option<&str>) -> &'static str {
    match last_auth {
        Some("AuthenticationSASL") => "SASLInitialResponse",
        Some("AuthenticationSASLContinue") => "SASLResponse",
        Some("AuthenticationGSS" | "AuthenticationGSSContinue" | "AuthenticationSSPI") => {
            "GSSResponse"
        }
        _ => "PasswordMessage",
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// One message of each type, with values that exercise each kind.
    pub(crate) fn samples() -> Vec<Msg> {
        let s = |x: &str| Val::Str(x.as_bytes().to_vec());
        let mut out = Vec::new();
        for spec in SPECS {
            let vals = spec
                .fields
                .iter()
                .map(|k| match k {
                    Byte => Val::Byte(if spec.name == "Message" { b'?' } else { b'I' }),
                    I8 => Val::Int(1),
                    I16 => Val::Int(-2),
                    I32 => Val::Int(16_385),
                    Version => Val::Int(196_610),
                    Str => s("a \"q\"\n\u{e9}"),
                    Rest => Val::Str(vec![0, 1, 255, b'x']),
                    Bytes32 => Val::Null,
                    Array16(g) | Array32(g) if g.len() == 1 => Val::List(vec![
                        match g[0] {
                            Bytes32 => s("v"),
                            Str => s("x"),
                            _ => Val::Int(7),
                        },
                        match g[0] {
                            Bytes32 => Val::Null,
                            Str => s("y"),
                            _ => Val::Int(8),
                        },
                    ]),
                    Array16(_) | Array32(_) => Val::List(vec![Val::Tuple(vec![
                        s("col"),
                        Val::Int(0),
                        Val::Int(0),
                        Val::Int(23),
                        Val::Int(4),
                        Val::Int(-1),
                        Val::Int(0),
                    ])]),
                    Fields => {
                        Val::List(vec![Val::Byte(b'S'), s("ERROR"), Val::Byte(b'C'), s("42601")])
                    }
                    Pairs => Val::List(vec![s("user"), s("u"), s("database"), s("d")]),
                    StrList => Val::List(vec![s("SCRAM-SHA-256"), s("SCRAM-SHA-256-PLUS")]),
                })
                .collect();
            out.push(Msg { spec, vals });
        }
        out
    }

    #[test]
    fn every_message_round_trips_through_its_frame() {
        for msg in samples() {
            let frame = msg.encode();
            let password = if msg.spec.key == Key::Password { msg.name() } else { "" };
            assert_eq!(Msg::decode(msg.dir(), &frame, password), msg, "{}", msg.name());
        }
    }

    #[test]
    fn a_bad_body_is_kept_raw() {
        let frame = Frame::Typed(b'Z', vec![b'I', b'x']);
        let msg = Msg::decode(Dir::B, &frame, "");
        assert_eq!(msg.name(), "Message");
        assert_eq!(msg.encode(), frame);
        let frame = Frame::Typed(b'#', vec![1, 2]);
        assert_eq!(Msg::decode(Dir::F, &frame, "").encode(), frame);
    }

    #[test]
    fn the_password_message_depends_on_the_exchange() {
        assert_eq!(password_name(Some("AuthenticationSASL")), "SASLInitialResponse");
        assert_eq!(password_name(Some("AuthenticationSASLContinue")), "SASLResponse");
        assert_eq!(password_name(Some("AuthenticationMD5Password")), "PasswordMessage");
    }

    #[test]
    fn names_are_unique_in_each_direction() {
        for a in SPECS {
            let same = SPECS.iter().filter(|b| b.dir == a.dir && b.name == a.name).count();
            assert_eq!(same, 1, "{}", a.name);
        }
    }
}
