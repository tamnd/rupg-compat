//! The statement count of the regression suite (spec/05 section 5.8): the statements of the suite whose output is equal to the output of the oracle.
//!
//! `pg_regress` runs psql with `-a`, so an output file has the lines of the SQL file, and the output of a statement comes after the line that ends it. psql does not echo an empty line outside of a quote or a comment, and it does not echo the data lines of `COPY ... FROM STDIN`. This module cuts the SQL file into units. A unit is the lines up to and with a line that ends one or more statements. The module finds the echoed lines of each unit in an output file, in order, and the output of the unit is the lines up to the next unit that it finds. The outputs are compared as `pg_regress` compares files, with `diff -w`, so white space in a line is not a difference.
//!
//! The count is for progress only. It is never used for a gate, because it hides a test that fails on its first statement.

use std::fs;
use std::ops::Range;
use std::path::Path;

/// Some lines of the SQL file that end one or more statements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Unit {
    pub(crate) lines: Range<usize>,
    /// The lines of the unit that psql echoes.
    pub(crate) echo: Vec<usize>,
    pub(crate) statements: usize,
}

/// The quote that the lexer is in.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Quote {
    /// A string in single quotes. The flag is true for an `E` string, where a backslash escapes the next character.
    Single(bool),
    Double,
    /// A string in dollar quotes, with its tag.
    Dollar(String),
}

/// The state of the statement lexer of psql (`psqlscan.l`), as far as it finds the end of a statement.
#[derive(Default)]
struct Lexer {
    quote: Option<Quote>,
    comment: usize,
    parens: usize,
    /// The depth of `BEGIN` and `END` in the body of `CREATE FUNCTION` or `CREATE PROCEDURE`.
    begin: usize,
    /// The first letters of the first four identifiers when they are `create`, `or`, `replace`, `function` or `procedure`, as psql keeps them.
    first: [u8; 4],
    /// The identifiers of the statement, in lower case.
    words: Vec<String>,
    word: String,
    /// True when the statement has text that is not white space or a comment.
    text: bool,
    /// The blocks of data lines of `COPY ... FROM STDIN` that are still to come. A query with `\\;` can have more than one.
    copy: usize,
}

impl Lexer {
    fn end_word(&mut self) {
        if self.word.is_empty() {
            return;
        }
        let w = std::mem::take(&mut self.word).to_lowercase();
        if w.as_bytes()[0].is_ascii_digit() {
            return;
        }
        let n = self.words.len();
        if n < 4 && ["create", "or", "replace", "function", "procedure"].contains(&w.as_str()) {
            self.first[n] = w.as_bytes()[0];
        }
        let f = self.first;
        let body = f[0] == b'c'
            && (f[1] == b'f'
                || f[1] == b'p'
                || (f[1] == b'o' && f[2] == b'r' && (f[3] == b'f' || f[3] == b'p')));
        if body && self.parens == 0 {
            match w.as_str() {
                "begin" => self.begin += 1,
                "case" if self.begin >= 1 => self.begin += 1,
                "end" => self.begin = self.begin.saturating_sub(1),
                _ => {}
            }
        }
        self.words.push(w);
    }

    /// Ends one statement of the query. When it is `COPY ... FROM STDIN`, a block of data lines follows the query. The grammar takes `FROM STDOUT` as `FROM STDIN`.
    fn end_part(&mut self) {
        self.end_word();
        let w = &self.words;
        if w.first().is_some_and(|w| w == "copy")
            && w.windows(2).any(|p| p[0] == "from" && (p[1] == "stdin" || p[1] == "stdout"))
        {
            self.copy += 1;
        }
        self.words.clear();
        self.first = [0; 4];
        self.begin = 0;
    }

    /// Ends the query that psql sends.
    fn end_statement(&mut self) {
        self.end_part();
        *self = Lexer { copy: self.copy, ..Lexer::default() };
    }

    /// Reads one line. Returns the number of statements that end in the line.
    fn line(&mut self, line: &str) -> usize {
        if self.copy > 0 {
            if line == "\\." {
                self.copy -= 1;
            }
            return 0;
        }
        let b = line.as_bytes();
        let mut ended = 0;
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            let next = b.get(i + 1).copied();
            match &self.quote {
                Some(Quote::Single(escape)) => {
                    if *escape && c == b'\\' {
                        i += 2;
                        continue;
                    }
                    if c == b'\'' {
                        if next == Some(b'\'') {
                            i += 2;
                            continue;
                        }
                        self.quote = None;
                    }
                    i += 1;
                    continue;
                }
                Some(Quote::Double) => {
                    if c == b'"' {
                        if next == Some(b'"') {
                            i += 2;
                            continue;
                        }
                        self.quote = None;
                    }
                    i += 1;
                    continue;
                }
                Some(Quote::Dollar(tag)) => {
                    if b[i..].starts_with(tag.as_bytes()) {
                        i += tag.len();
                        self.quote = None;
                    } else {
                        i += 1;
                    }
                    continue;
                }
                None => {}
            }
            if self.comment > 0 {
                if c == b'/' && next == Some(b'*') {
                    self.comment += 1;
                    i += 2;
                } else if c == b'*' && next == Some(b'/') {
                    self.comment -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            match c {
                b'-' if next == Some(b'-') => break,
                b'/' if next == Some(b'*') => {
                    self.end_word();
                    self.comment = 1;
                    i += 2;
                    continue;
                }
                b'\'' => {
                    let escape = self.word.eq_ignore_ascii_case("e");
                    if escape {
                        self.word.clear();
                    }
                    self.end_word();
                    self.quote = Some(Quote::Single(escape));
                    self.text = true;
                }
                b'"' => {
                    self.end_word();
                    self.quote = Some(Quote::Double);
                    self.text = true;
                }
                b'$' if !self.word.is_empty() => self.word.push('$'),
                b'$' => {
                    let tag = dollar_tag(&b[i..]);
                    self.text = true;
                    if let Some(tag) = tag {
                        i += tag.len();
                        self.quote = Some(Quote::Dollar(tag));
                        continue;
                    }
                }
                b';' => {
                    self.end_word();
                    self.text = true;
                    if self.parens == 0 && self.begin == 0 {
                        ended += 1;
                        self.end_statement();
                    }
                }
                // `\\;` puts a semicolon in the query and does not send it.
                b'\\' if next == Some(b';') => {
                    self.end_part();
                    self.text = true;
                    i += 2;
                    continue;
                }
                b'\\' => {
                    // A backslash command takes the rest of the line. A query before it runs with it, as with `\g` and `\gset`, so the two are one statement.
                    self.end_statement();
                    ended += 1;
                    let rest = line[i..].to_lowercase();
                    self.copy =
                        usize::from(rest.starts_with("\\copy") && rest.contains(" from stdin"));
                    break;
                }
                b'(' => {
                    self.end_word();
                    self.parens += 1;
                    self.text = true;
                }
                b')' => {
                    self.end_word();
                    self.parens = self.parens.saturating_sub(1);
                    self.text = true;
                }
                c if c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80 => {
                    // A UTF-8 sequence goes into the word byte by byte, so take the whole character.
                    let ch = line[i..].chars().next().expect("a character");
                    self.word.push(ch);
                    self.text = true;
                    i += ch.len_utf8();
                    continue;
                }
                c if c.is_ascii_whitespace() => self.end_word(),
                _ => {
                    self.end_word();
                    self.text = true;
                }
            }
            i += 1;
        }
        self.end_word();
        ended
    }
}

/// The tag of a dollar quote at the start of `b`, with both dollar signs, as `$$` or `$tag$`. A parameter such as `$1` is not a tag.
fn dollar_tag(b: &[u8]) -> Option<String> {
    let mut j = 1;
    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] >= 0x80) {
        if j == 1 && b[j].is_ascii_digit() {
            return None;
        }
        j += 1;
    }
    (b.get(j) == Some(&b'$')).then(|| String::from_utf8_lossy(&b[..=j]).into_owned())
}

/// Cuts a SQL file into units. Text after the last statement is a unit when psql sends it at the end of the file.
pub(crate) fn units(sql: &str) -> Vec<Unit> {
    let mut out = Vec::new();
    let mut lexer = Lexer::default();
    let mut start = 0;
    let mut echo = Vec::new();
    let mut pending = 0;
    let mut n = 0;
    for (k, line) in sql.lines().enumerate() {
        let in_copy = lexer.copy > 0;
        if !in_copy && (!line.is_empty() || lexer.quote.is_some() || lexer.comment > 0) {
            echo.push(k);
        }
        pending += lexer.line(line);
        n = k + 1;
        if (pending > 0 || in_copy) && lexer.copy == 0 {
            out.push(Unit {
                lines: start..n,
                echo: std::mem::take(&mut echo),
                statements: pending.max(1),
            });
            start = n;
            pending = 0;
        }
    }
    if lexer.text || pending > 0 {
        out.push(Unit { lines: start..n, echo, statements: pending.max(1) });
    }
    out
}

/// The first place at or after `from` where `block` is in `out`.
fn find(out: &[&str], from: usize, block: &[&str]) -> Option<usize> {
    if block.len() > out.len() {
        return None;
    }
    (from..=out.len() - block.len()).find(|&i| out[i..i + block.len()] == *block)
}

/// The output of each unit in an output file, or None for a unit whose lines are not in it.
fn outputs<'a>(sql: &[&str], units: &[Unit], out: &'a [&'a str]) -> Vec<Option<&'a [&'a str]>> {
    let mut starts = Vec::with_capacity(units.len());
    let mut at = 0;
    for u in units {
        let block: Vec<&str> = u.echo.iter().map(|&i| sql[i]).collect();
        let start = find(out, at, &block);
        if let Some(s) = start {
            at = s + block.len();
        }
        starts.push(start);
    }
    (0..units.len())
        .map(|k| {
            let s = starts[k]? + units[k].echo.len();
            let end = starts[k + 1..].iter().flatten().next().copied().unwrap_or(out.len());
            Some(&out[s..end])
        })
        .collect()
}

/// True when two outputs are equal with the white space in each line taken out, as `diff -w` compares them.
fn same(a: &[&str], b: &[&str]) -> bool {
    let squeeze = |l: &&str| l.split_whitespace().collect::<String>();
    a.len() == b.len() && a.iter().map(squeeze).eq(b.iter().map(squeeze))
}

/// The statement count of one test: the statements whose output is equal on both sides, and the statements whose lines are in the output of the oracle. Each side has its own SQL file, because `pg_regress` writes the paths of its output directory into the files that it makes from `input/`.
pub(crate) fn count(
    oracle_sql: &str,
    oracle: &str,
    server_sql: &str,
    server: Option<&str>,
) -> (usize, usize) {
    let (o_lines, s_lines): (Vec<&str>, Vec<&str>) =
        (oracle_sql.lines().collect(), server_sql.lines().collect());
    let (o_units, s_units) = (units(oracle_sql), units(server_sql));
    let o_out: Vec<&str> = oracle.lines().collect();
    let s_out: Vec<&str> = server.unwrap_or_default().lines().collect();
    let o = outputs(&o_lines, &o_units, &o_out);
    let s = if o_units.len() == s_units.len() {
        outputs(&s_lines, &s_units, &s_out)
    } else {
        vec![None; o_units.len()]
    };
    let mut n = (0, 0);
    for ((u, o), s) in o_units.iter().zip(&o).zip(&s) {
        if let Some(o) = o {
            n.1 += u.statements;
            if s.is_some_and(|s| same(o, s)) {
                n.0 += u.statements;
            }
        }
    }
    n
}

/// The SQL file of a test: the file that `pg_regress` made from `input/` in the output directory, or else the file of `sql/` in the directory of the suite.
fn sql_file(outdir: &Path, suite: &Path, test: &str) -> Option<(String, String)> {
    let dir = outdir.display().to_string();
    let name = format!("{test}.sql");
    let text =
        read(&outdir.join("sql").join(&name)).or_else(|| read(&suite.join("sql").join(&name)))?;
    Some((text, dir))
}

/// A file as text. Some tests are not in UTF-8, so a byte that is not UTF-8 becomes U+FFFD on both sides.
fn read(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// The statement count of a run of the regression suite against the run of the oracle. `oracle` and `server` are the output directories of `pg_regress` and `suite` is the directory of the suite. A test with no output of the oracle is not counted. The path of each output directory is replaced, so that the paths in the files of `input/` and `output/` do not differ.
pub(crate) fn suite_count(
    suite: &Path,
    oracle: &Path,
    server: &Path,
    tests: &[String],
) -> (usize, usize) {
    let mut n = (0, 0);
    for t in tests {
        let read = |dir: &Path| read(&dir.join("results").join(format!("{t}.out")));
        let (Some(o_out), Some((o_sql, o_dir)), Some((s_sql, s_dir))) =
            (read(oracle), sql_file(oracle, suite, t), sql_file(server, suite, t))
        else {
            continue;
        };
        let plain = |text: &str, dir: &str| text.replace(dir, "@outputdir@");
        let s_out = read(server).map(|s| plain(&s, &s_dir));
        let c = count(
            &plain(&o_sql, &o_dir),
            &plain(&o_out, &o_dir),
            &plain(&s_sql, &s_dir),
            s_out.as_deref(),
        );
        n.0 += c.0;
        n.1 += c.1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ends(sql: &str) -> Vec<(usize, usize, usize)> {
        units(sql).into_iter().map(|u| (u.lines.start, u.lines.end, u.statements)).collect()
    }

    #[test]
    fn a_unit_ends_at_the_line_with_the_semicolon() {
        let sql = "-- a comment\nSELECT 1;\n\nSELECT 'a;b',\n  \"c;\" -- d;\n  FROM t; SELECT 2;\n";
        assert_eq!(ends(sql), [(0, 2, 1), (2, 6, 2)]);
    }

    #[test]
    fn quotes_and_comments_hide_the_semicolon() {
        let sql = "SELECT E'\\';', $$;$$, $f$ ; $f$, /* ; /* ; */ ; */ 1;\nSELECT $1;\n";
        assert_eq!(ends(sql), [(0, 1, 1), (1, 2, 1)]);
    }

    #[test]
    fn a_body_in_begin_atomic_ends_at_its_end() {
        let sql = "CREATE OR REPLACE FUNCTION f() RETURNS int\nBEGIN ATOMIC\n  SELECT CASE WHEN true THEN 1 END;\n  SELECT 2;\nEND;\nSELECT f();\n";
        assert_eq!(ends(sql), [(0, 5, 1), (5, 6, 1)]);
    }

    #[test]
    fn copy_data_and_backslash_commands_end_units() {
        let sql = "COPY t FROM stdin;\n1;2\n\\.\n\\d t\nSELECT 1 \\gset\nSELECT 0\\; COPY t FROM stdin\\; COPY t FROM stdin;\n1\n\\.\n2\n\\.\nSELECT 2\n";
        assert_eq!(ends(sql), [(0, 3, 1), (3, 4, 1), (4, 5, 1), (5, 10, 1), (10, 11, 1)]);
    }

    #[test]
    fn the_count_compares_the_output_of_each_statement() {
        let sql = "SELECT 1;\n\n-- next\nSELECT 2;\nCOPY t FROM stdin;\n1\n\\.\nSELECT 3;\n";
        // psql echoes neither the empty line nor the data of `COPY`.
        let oracle = "SELECT 1;\n ?column? \n----------\n        1\n(1 row)\n\n-- next\nSELECT 2;\n x\n---\n 2\n(1 row)\n\nCOPY t FROM stdin;\nSELECT 3;\n";
        // White space in a line is not a difference. The second statement gives an error.
        let server = "SELECT 1;\n ?column?\n----------\n 1\n(1 row)\n\n-- next\nSELECT 2;\nERROR:  no\nCOPY t FROM stdin;\nSELECT 3;\n";
        assert_eq!(count(sql, oracle, sql, Some(oracle)), (4, 4));
        assert_eq!(count(sql, oracle, sql, Some(server)), (3, 4));
        // A server that stops after the first statement.
        assert_eq!(count(sql, oracle, sql, Some("SELECT 1;\nFATAL:  gone\n")), (0, 4));
        assert_eq!(count(sql, oracle, sql, None), (0, 4));
    }
}
