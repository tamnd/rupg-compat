//! The sqllogictest corpus of SQLite (spec/21 section 21.4.4). The harness runs each record of a file on the oracle and on the server under test, and compares the answers of the two servers. It does not use the expected answers of the corpus, because they are the answers of SQLite.
//!
//! The corpus comes from the repository and the commit of `[sqllogictest]` in `pins.toml`. A record has the conditions `skipif` and `onlyif` of the engine `postgresql`. A file passes when each of its records gives the answer of the oracle.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::client::{self, Conn, Target};
use crate::message::{Msg, Val};

/// The engine name of the conditions `skipif` and `onlyif`.
pub(crate) const ENGINE: &str = "postgresql";

/// How the rows of a query are put in order before they are compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sort {
    /// `nosort`: the order of the server.
    No,
    /// `rowsort`: the rows in text order.
    Row,
    /// `valuesort`: all the values in text order, with no rows.
    Value,
}

/// One record of a test file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Record {
    /// The line of `statement` or `query`, from 1.
    pub(crate) line: usize,
    /// `None` for a statement, else the sort mode of the query.
    pub(crate) query: Option<Sort>,
    pub(crate) sql: String,
}

/// Reads the records of a test file. A record that a condition skips for the engine `postgresql` is not in the list. `halt` stops the file, and `hash-threshold` has no effect here.
pub(crate) fn parse(text: &str) -> Result<Vec<Record>, String> {
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let mut records = Vec::new();
    let mut skip = false;
    let mut i = 0;
    while i < lines.len() {
        let line = i + 1;
        let words: Vec<&str> = lines[i].split_whitespace().collect();
        i += 1;
        match words.first().copied() {
            None => skip = false,
            Some(w) if w.starts_with('#') => {}
            Some("skipif") => skip |= words.get(1) == Some(&ENGINE),
            Some("onlyif") => skip |= words.get(1) != Some(&ENGINE),
            Some("hash-threshold") => {}
            Some("halt") if skip => skip = false,
            Some("halt") => break,
            Some(kind @ ("statement" | "query")) => {
                let query = (kind == "query").then(|| match words.get(2).copied() {
                    Some("rowsort") => Sort::Row,
                    Some("valuesort") => Sort::Value,
                    _ => Sort::No,
                });
                let start = i;
                while i < lines.len() && !lines[i].is_empty() && lines[i] != "----" {
                    i += 1;
                }
                let sql = lines[start..i].join("\n");
                // The expected answer of SQLite goes to the next blank line.
                while i < lines.len() && !lines[i].is_empty() {
                    i += 1;
                }
                if sql.is_empty() {
                    return Err(format!("line {line}: the record has no SQL"));
                }
                if !skip {
                    records.push(Record { line, query, sql });
                }
                skip = false;
            }
            Some(w) => return Err(format!("line {line}: unknown record {w:?}")),
        }
    }
    Ok(records)
}

/// The answer of one record in a form that compares: the error, the command tag of a statement, or the rows of a query in the order of its sort mode. A value is quoted, so that NULL and the text `NULL` are different.
pub(crate) fn answer(query: Option<Sort>, msgs: &[Msg]) -> String {
    if let Some(e) = msgs.iter().find(|m| m.name() == "ErrorResponse") {
        return format!("error {}", client::error_text(e));
    }
    let Some(sort) = query else {
        let tag = msgs
            .iter()
            .find(|m| m.name() == "CommandComplete")
            .and_then(|m| m.vals.first()?.bytes())
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default();
        return format!("ok {tag}");
    };
    let value = |v: &Val| match v.bytes() {
        Some(b) => format!("{:?}", String::from_utf8_lossy(b)),
        None => "NULL".to_string(),
    };
    let rows: Vec<Vec<String>> = msgs
        .iter()
        .filter(|m| m.name() == "DataRow")
        .map(|m| m.vals.first().map(Val::list).unwrap_or_default().iter().map(value).collect())
        .collect();
    let mut lines: Vec<String> = match sort {
        Sort::Value => rows.into_iter().flatten().collect(),
        _ => rows.iter().map(|r| r.join(" ")).collect(),
    };
    if sort != Sort::No {
        lines.sort();
    }
    let count = lines.len();
    format!("{count} lines\n{}", lines.join("\n"))
}

/// The test files of the corpus under `paths`, relative to the `test` directory of the corpus, in name order. A path is a file or a directory.
pub(crate) fn test_files(root: &Path, paths: &[String]) -> Result<Vec<String>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out)?;
            } else if path.extension().is_some_and(|e| e == "test") {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut found = Vec::new();
    for p in paths {
        let path = root.join(p);
        if path.is_dir() {
            walk(&path, &mut found)?;
        } else if path.is_file() {
            found.push(path);
        } else {
            return Err(format!("{}: no such file or directory in the corpus", path.display()));
        }
    }
    let mut names: Vec<String> = found
        .iter()
        .filter_map(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    names.sort();
    names.dedup();
    Ok(names)
}

/// Fetches the corpus at the pin into `<work>/sqllogictest/<commit>` and returns its `test` directory. A checkout that is at the pin is used again.
pub(crate) fn fetch(work: &Path, repository: &str, commit: &str) -> Result<PathBuf, String> {
    let dir = work.join("sqllogictest").join(commit);
    let git = |args: &[&str]| -> Result<String, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(args)
            .output()
            .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
        if !out.status.success() {
            return Err(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    if git(&["rev-parse", "HEAD"]).ok().as_deref() != Some(commit) {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        git(&["init", "-q"])?;
        git(&["fetch", "-q", "--depth", "1", repository, commit])?;
        git(&["checkout", "-q", "-f", "--detach", "FETCH_HEAD"])?;
        let head = git(&["rev-parse", "HEAD"])?;
        if head != commit {
            return Err(format!("the corpus is at {head}, not at the pin {commit}"));
        }
    }
    Ok(dir.join("test"))
}

/// The two servers of a run. When the server under test is the oracle, the two sides use different databases.
pub(crate) struct Sides {
    pub(crate) oracle: Target,
    pub(crate) server: Target,
    /// The time that the server under test has for one record.
    pub(crate) timeout: Duration,
}

/// The result of one file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileResult {
    pub(crate) name: String,
    pub(crate) records: usize,
    pub(crate) equal: usize,
    /// The first record that is different: its line, the answer of the oracle and the answer of the server under test.
    pub(crate) first_diff: Option<(usize, String, String)>,
}

impl FileResult {
    pub(crate) fn pass(&self) -> bool {
        self.equal == self.records
    }
}

/// Runs each file on `jobs` connections of each side in parallel. Worker `n` uses the database `slt_n` on each side, which it makes again for each file.
pub(crate) fn run(
    root: &Path,
    files: &[String],
    sides: &Sides,
    jobs: usize,
) -> Result<Vec<FileResult>, String> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::new());
    let failure = Mutex::new(None);
    std::thread::scope(|scope| {
        for worker in 0..jobs.max(1) {
            let (next, results, failure) = (&next, &results, &failure);
            scope.spawn(move || {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(name) = files.get(i) else { break };
                    match run_file(root, name, sides, worker) {
                        Ok(r) => results.lock().unwrap_or_else(|e| e.into_inner()).push(r),
                        Err(e) => {
                            *failure.lock().unwrap_or_else(|e| e.into_inner()) =
                                Some(format!("{name}: {e}"));
                            next.store(files.len(), Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });
        }
    });
    if let Some(e) = failure.into_inner().unwrap_or_else(|e| e.into_inner()) {
        return Err(e);
    }
    let mut results = results.into_inner().unwrap_or_else(|e| e.into_inner());
    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Runs one file. An error of the oracle stops the run. When the server under test closes the connection or does not answer in time, the records that are left are not equal.
fn run_file(root: &Path, name: &str, sides: &Sides, worker: usize) -> Result<FileResult, String> {
    let text = std::fs::read_to_string(root.join(name)).map_err(|e| e.to_string())?;
    let records = parse(&text)?;
    let same = sides.oracle.addr == sides.server.addr;
    let oracle_db = format!("slt_{worker}");
    let server_db = if same { format!("slt_{worker}_server") } else { oracle_db.clone() };
    client::reset_database(&sides.oracle, &oracle_db)?;
    let mut oracle = Conn::connect(&sides.oracle, &oracle_db)?;
    let mut server = client::reset_database(&sides.server, &server_db)
        .and_then(|()| Conn::connect(&sides.server, &server_db));
    if let Ok(c) = &server {
        c.set_timeout(sides.timeout)?;
    }
    let mut result =
        FileResult { name: name.to_string(), records: records.len(), ..Default::default() };
    for r in &records {
        let want = answer(r.query, &oracle.query(&r.sql)?);
        let got = match &mut server {
            Ok(c) => match c.query(&r.sql) {
                Ok(msgs) => answer(r.query, &msgs),
                Err(e) => {
                    server = Err(e.clone());
                    format!("no answer: {e}")
                }
            },
            Err(e) => format!("no answer: {e}"),
        };
        if want == got {
            result.equal += 1;
        } else if result.first_diff.is_none() {
            result.first_diff = Some((r.line, want, got));
        }
    }
    drop(oracle);
    drop(server);
    Conn::connect(&sides.oracle, "template1")?
        .execute(&format!("DROP DATABASE IF EXISTS {oracle_db} WITH (FORCE)"))?;
    if let Ok(mut c) = Conn::connect(&sides.server, "template1") {
        let _ = c.execute(&format!("DROP DATABASE IF EXISTS {server_db} WITH (FORCE)"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Dir;

    #[test]
    fn the_parser_reads_statements_and_queries() {
        let text = "hash-threshold 8\n\nstatement ok\nCREATE TABLE t1(a INTEGER, b INTEGER)\n\n# a comment\nquery II rowsort label-1\nSELECT a,\n b FROM t1\n----\n1\n2\n\nquery I nosort\nSELECT 1\n\nstatement error\nSELECT x\n";
        let records = parse(text).unwrap();
        assert_eq!(records.len(), 4);
        assert_eq!(
            records[0],
            Record { line: 3, query: None, sql: "CREATE TABLE t1(a INTEGER, b INTEGER)".into() }
        );
        assert_eq!(
            records[1],
            Record { line: 7, query: Some(Sort::Row), sql: "SELECT a,\n b FROM t1".into() }
        );
        assert_eq!(records[2].query, Some(Sort::No));
        assert_eq!(records[2].sql, "SELECT 1");
        assert_eq!(records[3].line, 17);
    }

    #[test]
    fn the_conditions_are_for_postgresql() {
        let text = "skipif postgresql\nstatement ok\nSELECT 1\n\nonlyif mysql\nstatement ok\nSELECT 2\n\nonlyif postgresql\nstatement ok\nSELECT 3\n\nskipif mysql\nstatement ok\nSELECT 4\n\nonlyif mssql\nhalt\n\nstatement ok\nSELECT 5\n\nhalt\n\nstatement ok\nSELECT 6\n";
        let sql: Vec<String> = parse(text).unwrap().into_iter().map(|r| r.sql).collect();
        assert_eq!(sql, ["SELECT 3", "SELECT 4", "SELECT 5"]);
    }

    #[test]
    fn an_unknown_record_is_an_error() {
        assert!(parse("statement ok\nSELECT 1\n\nloop i 1 10\n").is_err());
    }

    fn row(values: &[Option<&str>]) -> Msg {
        let vals = values
            .iter()
            .map(|v| match v {
                Some(s) => Val::Str(s.as_bytes().to_vec()),
                None => Val::Null,
            })
            .collect();
        Msg::new(Dir::B, "DataRow", vec![Val::List(vals)])
    }

    #[test]
    fn the_answer_follows_the_sort_mode() {
        let msgs = [row(&[Some("2"), None]), row(&[Some("1"), Some("NULL")])];
        assert_eq!(answer(Some(Sort::No), &msgs), "2 lines\n\"2\" NULL\n\"1\" \"NULL\"");
        assert_eq!(answer(Some(Sort::Row), &msgs), "2 lines\n\"1\" \"NULL\"\n\"2\" NULL");
        assert_eq!(answer(Some(Sort::Value), &msgs), "4 lines\n\"1\"\n\"2\"\n\"NULL\"\nNULL");
    }

    #[test]
    fn a_statement_gives_its_tag() {
        let done = Msg::new(Dir::B, "CommandComplete", vec![client::str_val("INSERT 0 1")]);
        assert_eq!(answer(None, &[done]), "ok INSERT 0 1");
    }
}
