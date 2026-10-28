//! The reducer of spec/21 section 21.3.7. It makes a failing SQL file smaller in three passes. The first pass removes statements with delta debugging. The second pass removes clauses, groups in parentheses, items of lists and tokens. The third pass makes literals smaller. A step stays only if the file still fails in the same way: the first statement with a different answer has the same first different message type and the same SQLSTATE on each side.

use std::time::Duration;

use crate::client::{self, Conn, Target};
use crate::message::Msg;

/// The keywords that start a clause that the second pass can remove.
const CLAUSES: [&str; 16] = [
    "WHERE",
    "GROUP",
    "HAVING",
    "ORDER",
    "LIMIT",
    "OFFSET",
    "RETURNING",
    "WINDOW",
    "UNION",
    "INTERSECT",
    "EXCEPT",
    "FOR",
    "ON",
    "USING",
    "SEARCH",
    "CYCLE",
];
/// The keywords that end an item of a list or a clause, with the keywords of `CLAUSES`.
const BOUNDS: [&str; 10] =
    ["SELECT", "FROM", "INTO", "VALUES", "SET", "JOIN", "WHEN", "THEN", "ELSE", "END"];

fn is_clause(t: &str) -> bool {
    CLAUSES.contains(&t.to_ascii_uppercase().as_str())
}

fn is_bound(t: &str) -> bool {
    is_clause(t) || BOUNDS.contains(&t.to_ascii_uppercase().as_str())
}

/// Splits a SQL file into statements at each `;` that is not in a quote or a comment. A line comment is removed.
pub(crate) fn split(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => {
                cur.push(c);
                for d in chars.by_ref() {
                    cur.push(d);
                    if d == c {
                        break;
                    }
                }
            }
            '-' if chars.peek() == Some(&'-') => {
                for d in chars.by_ref() {
                    if d == '\n' {
                        cur.push('\n');
                        break;
                    }
                }
            }
            ';' => {
                let s = cur.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    let s = cur.trim();
    if !s.is_empty() {
        out.push(s.to_string());
    }
    out
}

/// The tokens of a statement: quoted strings and names, words and numbers, parentheses, commas, and runs of other characters.
pub(crate) fn tokens(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = sql.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '\'' || c == '"' {
            let mut t = String::new();
            t.push(c);
            chars.next();
            while let Some(d) = chars.next() {
                t.push(d);
                // A doubled quote is a quote in the string.
                if d == c && chars.peek() != Some(&c) {
                    break;
                }
                if d == c {
                    t.push(c);
                    chars.next();
                }
            }
            out.push(t);
        } else if c.is_alphanumeric() || c == '_' || c == '$' || c == '.' {
            let mut t = String::new();
            while let Some(&d) = chars.peek() {
                if !(d.is_alphanumeric() || d == '_' || d == '$' || d == '.') {
                    break;
                }
                t.push(d);
                chars.next();
            }
            // A string with a prefix, such as E'a' or B'101', is one token.
            if let Some(&q) = chars.peek()
                && q == '\''
                && t.len() <= 2
                && t.chars().all(|x| x.is_ascii_alphabetic() || x == '&')
            {
                let rest = tokens(&chars.clone().collect::<String>());
                if let Some(first) = rest.first() {
                    t.push_str(first);
                    for _ in 0..first.chars().count() {
                        chars.next();
                    }
                }
            }
            out.push(t);
        } else if c == '(' || c == ')' || c == ',' || c == '[' || c == ']' {
            out.push(c.to_string());
            chars.next();
        } else {
            let mut t = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_whitespace() || d.is_alphanumeric() || "_$.'\"(),[]".contains(d) {
                    break;
                }
                t.push(d);
                chars.next();
            }
            out.push(t);
        }
    }
    out
}

/// The text of a list of tokens.
pub(crate) fn join(tokens: &[String]) -> String {
    tokens.join(" ")
}

/// The index of the token that closes the group that opens at `open`.
fn close_of(tokens: &[String], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, t) in tokens.iter().enumerate().skip(open) {
        match t.as_str() {
            "(" | "[" => depth += 1,
            ")" | "]" => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// The smaller forms of a statement for the second pass, from the largest change to the smallest.
pub(crate) fn smaller(tokens: &[String]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let with = |from: usize, to: usize, put: &[&str]| {
        let mut v: Vec<String> = tokens[..from].to_vec();
        v.extend(put.iter().map(|s| (*s).to_string()));
        v.extend(tokens[to..].iter().cloned());
        v
    };
    // The depth of each token, to find the end of a clause or of an item of a list.
    let mut depth = Vec::with_capacity(tokens.len());
    let mut d = 0i32;
    for t in tokens {
        if t == ")" || t == "]" {
            d -= 1;
        }
        depth.push(d);
        if t == "(" || t == "[" {
            d += 1;
        }
    }
    let end_at = |start: usize, stop: &dyn Fn(&str) -> bool| {
        let mut i = start + 1;
        while i < tokens.len() && depth[i] >= depth[start] {
            if depth[i] == depth[start] && stop(&tokens[i]) {
                break;
            }
            i += 1;
        }
        i
    };
    for (i, t) in tokens.iter().enumerate() {
        if is_clause(t) {
            let end = end_at(i, &is_bound);
            out.push(with(i, end, &[]));
        }
    }
    for (i, t) in tokens.iter().enumerate() {
        if (t == "(" || t == "[")
            && let Some(c) = close_of(tokens, i)
        {
            out.push(with(i, c + 1, &[]));
            if c > i + 1 {
                let inner: Vec<&str> = tokens[i + 1..c].iter().map(String::as_str).collect();
                out.push(with(i, c + 1, &inner));
                out.push(with(i, c + 1, &["NULL"]));
                out.push(with(i + 1, c, &[]));
            }
        }
    }
    for (i, t) in tokens.iter().enumerate() {
        if t == "," {
            // The item after the comma, and the item before it.
            let end = end_at(i, &|s: &str| s == "," || is_bound(s));
            out.push(with(i, end, &[]));
            let mut start = i;
            while start > 0
                && depth[start - 1] >= depth[i]
                && !(depth[start - 1] == depth[i]
                    && (tokens[start - 1] == "," || is_bound(&tokens[start - 1])))
            {
                start -= 1;
            }
            out.push(with(start, i + 1, &[]));
        }
    }
    for i in 0..tokens.len() {
        out.push(with(i, i + 1, &[]));
    }
    out.retain(|v| !v.is_empty() && v.len() < tokens.len());
    out
}

/// The smaller literals of a statement for the third pass.
pub(crate) fn literals(tokens: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let options: &[&str] = if t.starts_with('\'') {
            &["''", "'a'"]
        } else if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            &["0", "1"]
        } else {
            &[]
        };
        for o in options {
            if o.len() < t.len() {
                let mut v = tokens.to_vec();
                v[i] = (*o).to_string();
                out.push(v);
            }
        }
    }
    out
}

/// The messages of one statement in a form that compares: the data rows in text order, and no `ReadyForQuery`.
fn normal(msgs: &[Msg]) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for m in msgs.iter().filter(|m| m.name() != "ReadyForQuery") {
        match m.name() {
            "DataRow" => rows.push(format!("{:?}", m.vals)),
            "ErrorResponse" | "NoticeResponse" => {
                out.push(format!("{} {}", m.name(), client::error_text(m)))
            }
            name => {
                if !rows.is_empty() {
                    rows.sort();
                    out.append(&mut rows);
                }
                out.push(format!("{name} {:?}", m.vals));
            }
        }
    }
    rows.sort();
    out.append(&mut rows);
    out
}

/// The way a statement fails: the first different message type and the SQLSTATE on each side, or `None` when the answers are the same.
fn first_difference(want: &[Msg], got: &[Msg]) -> Option<String> {
    let (a, b) = (normal(want), normal(got));
    if a == b {
        return None;
    }
    let i = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let w: Vec<&Msg> = want.iter().filter(|m| m.name() != "ReadyForQuery").collect();
    let g: Vec<&Msg> = got.iter().filter(|m| m.name() != "ReadyForQuery").collect();
    let name = |list: &[&Msg]| list.get(i).map_or("(no message)", |m| m.name()).to_string();
    let state = |list: &[&Msg]| {
        list.iter()
            .find(|m| m.name() == "ErrorResponse")
            .map(|e| client::error_text(e).split(' ').nth(1).unwrap_or_default().to_string())
            .unwrap_or_default()
    };
    Some(format!("oracle {} {} | server {} {}", name(&w), state(&w), name(&g), state(&g)))
}

/// Runs files on the oracle and on the server under test, and counts the runs.
pub(crate) struct Tester {
    pub(crate) oracle: Target,
    pub(crate) server: Target,
    pub(crate) timeout: Duration,
    pub(crate) runs: usize,
    pub(crate) max_runs: usize,
}

impl Tester {
    fn side(target: &Target, db: &str, timeout: Duration) -> Result<Conn, String> {
        client::reset_database(target, db)?;
        let c = Conn::connect(target, db)?;
        c.set_timeout(timeout)?;
        Ok(c)
    }

    /// The way the statements fail, or `None` when each statement gives the answer of the oracle.
    pub(crate) fn signature(&mut self, stmts: &[String]) -> Result<Option<String>, String> {
        self.runs += 1;
        let same = self.oracle.addr == self.server.addr;
        let server_db = if same { "reduce_server" } else { "reduce" };
        let mut o = Tester::side(&self.oracle, "reduce_oracle", Duration::from_secs(60))?;
        let mut s = Tester::side(&self.server, server_db, self.timeout);
        o.query("SET statement_timeout = '5s'")?;
        if let Ok(c) = &mut s {
            let _ = c.query("SET statement_timeout = '5s'");
        }
        for (n, sql) in stmts.iter().enumerate() {
            let want = o.query(sql)?;
            let got = match &mut s {
                Ok(c) => c.query(sql),
                Err(e) => Err(e.clone()),
            };
            let got = match got {
                Ok(m) => m,
                Err(_) => return Ok(Some(format!("statement {n}: the server gives no answer"))),
            };
            if let Some(sig) = first_difference(&want, &got) {
                return Ok(Some(sig));
            }
        }
        Ok(None)
    }

    fn keeps(&mut self, stmts: &[String], sig: &str) -> Result<bool, String> {
        if self.runs >= self.max_runs {
            return Ok(false);
        }
        Ok(self.signature(stmts)?.is_some_and(|s| without_number(&s) == without_number(sig)))
    }
}

/// A signature without the number of the statement, because the first pass removes statements.
fn without_number(sig: &str) -> &str {
    sig.split_once(": ").map_or(sig, |(_, rest)| rest)
}

/// Removes statements with delta debugging.
fn statements(t: &mut Tester, list: &mut Vec<String>, sig: &str) -> Result<(), String> {
    let mut n = 2;
    while list.len() >= 2 {
        let chunk = list.len().div_ceil(n);
        let mut cut = false;
        let mut start = 0;
        while start < list.len() {
            let mut rest = list[..start].to_vec();
            rest.extend(list[(start + chunk).min(list.len())..].iter().cloned());
            if !rest.is_empty() && t.keeps(&rest, sig)? {
                *list = rest;
                n = (n - 1).max(2);
                cut = true;
                break;
            }
            start += chunk;
        }
        if !cut {
            if n >= list.len() {
                break;
            }
            n = (n * 2).min(list.len());
        }
    }
    Ok(())
}

/// Applies the first change of `changes` that keeps the signature, until no change keeps it.
fn each_statement(
    t: &mut Tester,
    list: &mut [String],
    sig: &str,
    changes: fn(&[String]) -> Vec<Vec<String>>,
) -> Result<(), String> {
    for i in 0..list.len() {
        loop {
            let toks = tokens(&list[i]);
            let mut changed = false;
            for c in changes(&toks) {
                let mut next = list.to_vec();
                next[i] = join(&c);
                if t.keeps(&next, sig)? {
                    list[i] = next[i].clone();
                    changed = true;
                    break;
                }
            }
            if !changed {
                break;
            }
        }
    }
    Ok(())
}

/// The result of a reduction.
pub(crate) struct Reduced {
    pub(crate) signature: String,
    pub(crate) statements: Vec<String>,
}

/// Reduces the statements in the three passes. It fails when the statements give the answers of the oracle.
pub(crate) fn reduce(t: &mut Tester, mut list: Vec<String>) -> Result<Reduced, String> {
    let signature = t
        .signature(&list)?
        .ok_or("each statement gives the answer of the oracle, so there is nothing to reduce")?;
    statements(t, &mut list, &signature)?;
    each_statement(t, &mut list, &signature, smaller)?;
    each_statement(t, &mut list, &signature, literals)?;
    // The second pass can make a statement that is not necessary.
    statements(t, &mut list, &signature)?;
    Ok(Reduced { signature, statements: list })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_statements_split_at_semicolons() {
        let text = "-- a comment; with a semicolon\nCREATE TABLE t (a text);\nINSERT INTO t VALUES ('a;b');\n\nSELECT \";\" FROM t";
        assert_eq!(
            split(text),
            ["CREATE TABLE t (a text)", "INSERT INTO t VALUES ('a;b')", "SELECT \";\" FROM t"]
        );
    }

    #[test]
    fn the_tokens_keep_quotes() {
        let t = tokens("SELECT a.b, 'it''s', E'x', count(*)::int8 FROM t WHERE a<=1");
        assert_eq!(
            t,
            [
                "SELECT", "a.b", ",", "'it''s'", ",", "E'x'", ",", "count", "(", "*", ")", "::",
                "int8", "FROM", "t", "WHERE", "a", "<=", "1"
            ]
        );
    }

    #[test]
    fn a_clause_and_a_group_can_go() {
        let t = tokens("SELECT a FROM t WHERE (a > 1) ORDER BY a");
        let all: Vec<String> = smaller(&t).iter().map(|v| join(v)).collect();
        assert!(all.contains(&"SELECT a FROM t ORDER BY a".to_string()));
        assert!(all.contains(&"SELECT a FROM t WHERE a > 1 ORDER BY a".to_string()));
        assert!(all.contains(&"SELECT a FROM t WHERE NULL ORDER BY a".to_string()));
        assert!(all.iter().all(|s| s.len() < join(&t).len() || s.contains("NULL")));
    }

    #[test]
    fn an_item_of_a_list_can_go() {
        let t = tokens("SELECT a, b + 1, c FROM t");
        let all: Vec<String> = smaller(&t).iter().map(|v| join(v)).collect();
        assert!(all.contains(&"SELECT a , c FROM t".to_string()));
        assert!(all.contains(&"SELECT a , b + 1 FROM t".to_string()));
        assert!(all.contains(&"SELECT b + 1 , c FROM t".to_string()));
    }

    #[test]
    fn literals_get_smaller() {
        let t = tokens("SELECT 12345, 'abc', 1");
        let all: Vec<String> = literals(&t).iter().map(|v| join(v)).collect();
        assert_eq!(
            all,
            [
                "SELECT 0 , 'abc' , 1",
                "SELECT 1 , 'abc' , 1",
                "SELECT 12345 , '' , 1",
                "SELECT 12345 , 'a' , 1"
            ]
        );
    }
}
