//! The first byte where the answers of two servers differ, for `rupg-compat diff` (spec/21 section 21.3.4).

use crate::message::Msg;

/// The place of the first different byte in two lists of backend messages.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Difference {
    /// The offset of the byte in the whole answer.
    pub(crate) offset: usize,
    /// The index of the message that has the byte.
    pub(crate) message: usize,
    /// The offset of the byte in that message, from its type byte.
    pub(crate) within: usize,
    /// The byte of each side, or None when that side has no byte there.
    pub(crate) want: Option<u8>,
    pub(crate) got: Option<u8>,
}

/// Finds the first byte where the wire encodings of `want` and `got` differ.
pub(crate) fn first_difference(want: &[Msg], got: &[Msg]) -> Option<Difference> {
    let mut offset = 0;
    for i in 0..want.len().max(got.len()) {
        let a = want.get(i).map(|m| m.encode().to_bytes()).unwrap_or_default();
        let b = got.get(i).map(|m| m.encode().to_bytes()).unwrap_or_default();
        let same = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
        if same < a.len().max(b.len()) {
            return Some(Difference {
                offset: offset + same,
                message: i,
                within: same,
                want: a.get(same).copied(),
                got: b.get(same).copied(),
            });
        }
        offset += a.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::str_val;
    use crate::message::{Dir, Val};

    fn done(tag: &str) -> Msg {
        Msg::new(Dir::B, "CommandComplete", vec![str_val(tag)])
    }

    fn ready() -> Msg {
        Msg::new(Dir::B, "ReadyForQuery", vec![Val::Byte(b'I')])
    }

    #[test]
    fn equal_answers_have_no_difference() {
        assert_eq!(
            first_difference(&[done("SELECT 1"), ready()], &[done("SELECT 1"), ready()]),
            None
        );
    }

    #[test]
    fn the_first_different_byte_is_found() {
        let d =
            first_difference(&[done("SELECT 1"), ready()], &[done("SELECT 2"), ready()]).unwrap();
        // 'C', four length bytes, then "SELECT " and the digit.
        assert_eq!(
            d,
            Difference { offset: 12, message: 0, within: 12, want: Some(b'1'), got: Some(b'2') }
        );
    }

    #[test]
    fn a_missing_message_is_a_difference() {
        let d = first_difference(&[done("SELECT 1"), ready()], &[done("SELECT 1")]).unwrap();
        assert_eq!((d.message, d.within, d.want, d.got), (1, 0, Some(b'Z'), None));
        assert_eq!(d.offset, done("SELECT 1").encode().to_bytes().len());
    }
}
