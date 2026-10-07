//! A reader for the subset of TOML that `pins.toml` and `ratchet.toml` use.
//!
//! The subset is: comments, `[table]` and `[table.sub]` headers, `key = value` lines, basic and literal strings, integers, booleans and arrays. Inline tables, dates, floats and dotted keys are not in the subset, and the reader rejects them. The harness has few dependencies, so it does not use a TOML crate for two small files.

use std::collections::BTreeMap;
use std::fmt;

/// One TOML value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    Array(Vec<Value>),
    Table(Table),
}

/// A table. The keys are in sorted order.
pub(crate) type Table = BTreeMap<String, Value>;

/// An error with the line number where it happened.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Error {
    pub(crate) line: usize,
    pub(crate) message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Value {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub(crate) fn as_table(&self) -> Option<&Table> {
        match self {
            Value::Table(t) => Some(t),
            _ => None,
        }
    }
}

/// Parses a document into its root table.
pub(crate) fn parse(text: &str) -> Result<Table, Error> {
    let mut root = Table::new();
    let mut path: Vec<String> = Vec::new();
    let mut headers: Vec<Vec<String>> = Vec::new();
    let mut p = Parser { chars: text.chars().collect(), pos: 0, line: 1 };
    loop {
        p.skip_space_and_comments(true);
        let Some(c) = p.peek() else { break };
        if c == '[' {
            p.pos += 1;
            path = p.header()?;
            if headers.contains(&path) {
                return Err(p.error(format!("table [{}] is defined twice", path.join("."))));
            }
            headers.push(path.clone());
            table_at(&mut root, &path, p.line)?;
        } else {
            let key = p.key()?;
            p.skip_space_and_comments(false);
            if p.next() != Some('=') {
                return Err(p.error(format!("expected `=` after key `{key}`")));
            }
            p.skip_space_and_comments(false);
            let value = p.value()?;
            let table = table_at(&mut root, &path, p.line)?;
            if table.insert(key.clone(), value).is_some() {
                return Err(p.error(format!("key `{key}` is defined twice")));
            }
        }
        p.end_of_line()?;
    }
    Ok(root)
}

fn table_at<'a>(root: &'a mut Table, path: &[String], line: usize) -> Result<&'a mut Table, Error> {
    let mut table = root;
    for name in path {
        let entry = table.entry(name.clone()).or_insert_with(|| Value::Table(Table::new()));
        table = match entry {
            Value::Table(t) => t,
            _ => return Err(Error { line, message: format!("`{name}` is not a table") }),
        };
    }
    Ok(table)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    fn error(&self, message: String) -> Error {
        Error { line: self.line, message }
    }

    /// Skips spaces and comments. With `newlines`, it also skips line ends.
    fn skip_space_and_comments(&mut self, newlines: bool) {
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => self.pos += 1,
                '\n' if newlines => {
                    self.next();
                }
                '#' => {
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn end_of_line(&mut self) -> Result<(), Error> {
        self.skip_space_and_comments(false);
        match self.next() {
            None | Some('\n') => Ok(()),
            Some(c) => Err(self.error(format!("unexpected `{c}` at the end of the line"))),
        }
    }

    fn header(&mut self) -> Result<Vec<String>, Error> {
        let mut path = Vec::new();
        loop {
            self.skip_space_and_comments(false);
            path.push(self.key()?);
            self.skip_space_and_comments(false);
            match self.next() {
                Some('.') => {}
                Some(']') => return Ok(path),
                _ => return Err(self.error("expected `.` or `]` in a table header".into())),
            }
        }
    }

    fn key(&mut self) -> Result<String, Error> {
        match self.peek() {
            Some('"') => {
                self.pos += 1;
                self.basic_string()
            }
            Some('\'') => {
                self.pos += 1;
                self.literal_string()
            }
            _ => {
                let start = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    self.pos += 1;
                }
                if start == self.pos {
                    return Err(self.error("expected a key".into()));
                }
                Ok(self.chars[start..self.pos].iter().collect())
            }
        }
    }

    fn value(&mut self) -> Result<Value, Error> {
        match self.peek() {
            Some('"') => {
                self.pos += 1;
                self.basic_string().map(Value::Str)
            }
            Some('\'') => {
                self.pos += 1;
                self.literal_string().map(Value::Str)
            }
            Some('[') => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_space_and_comments(true);
                    if self.peek() == Some(']') {
                        self.pos += 1;
                        return Ok(Value::Array(items));
                    }
                    items.push(self.value()?);
                    self.skip_space_and_comments(true);
                    match self.next() {
                        Some(',') => {}
                        Some(']') => return Ok(Value::Array(items)),
                        _ => return Err(self.error("expected `,` or `]` in an array".into())),
                    }
                }
            }
            _ => {
                let start = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || "+-_.:".contains(c))
                {
                    self.pos += 1;
                }
                let word: String = self.chars[start..self.pos].iter().collect();
                match word.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => word
                        .replace('_', "")
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| self.error(format!("value `{word}` is not in the subset"))),
                }
            }
        }
    }

    fn basic_string(&mut self) -> Result<String, Error> {
        let mut out = String::new();
        loop {
            match self.next() {
                None | Some('\n') => return Err(self.error("unterminated string".into())),
                Some('"') => return Ok(out),
                Some('\\') => {
                    let c = match self.next() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('"') => '"',
                        Some('\\') => '\\',
                        Some('u') => {
                            let hex: String = (0..4).filter_map(|_| self.next()).collect();
                            u32::from_str_radix(&hex, 16)
                                .ok()
                                .and_then(char::from_u32)
                                .ok_or_else(|| self.error(format!("bad escape \\u{hex}")))?
                        }
                        other => return Err(self.error(format!("bad escape {other:?}"))),
                    };
                    out.push(c);
                }
                Some(c) => out.push(c),
            }
        }
    }

    fn literal_string(&mut self) -> Result<String, Error> {
        let mut out = String::new();
        loop {
            match self.next() {
                None | Some('\n') => return Err(self.error("unterminated string".into())),
                Some('\'') => return Ok(out),
                Some(c) => out.push(c),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_keys_and_values() {
        let doc = r#"
# a comment
top = 1

[a]
s = "x \"y\" \u00e9" # trailing comment
l = 'C:\path'
n = -1_000
flag = true
list = [
  "one", # first
  "two",
]

[a.b]
"quoted key" = []
"#;
        let t = parse(doc).unwrap();
        assert_eq!(t["top"], Value::Int(1));
        let a = t["a"].as_table().unwrap();
        assert_eq!(a["s"].as_str(), Some("x \"y\" \u{e9}"));
        assert_eq!(a["l"].as_str(), Some("C:\\path"));
        assert_eq!(a["n"].as_int(), Some(-1000));
        assert_eq!(a["flag"], Value::Bool(true));
        assert!(matches!(&a["list"], Value::Array(v) if v.len() == 2));
        let b = a["b"].as_table().unwrap();
        assert_eq!(b["quoted key"], Value::Array(vec![]));
    }

    #[test]
    fn errors_name_the_line() {
        let e = parse("a = 1\na = 2\n").unwrap_err();
        assert_eq!(e.line, 2);
        assert!(parse("[x]\n[x]\n").is_err());
        assert!(parse("a = 1.5\n").is_err());
        assert!(parse("a = \"open\n").is_err());
        assert!(parse("a = 1 b\n").is_err());
        assert!(parse("a = { b = 1 }\n").is_err());
    }
}
