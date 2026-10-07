//! The reference outputs of the shim suites (spec/03 section 3.15, item 7).
//!
//! Each shim version 14 to 18 keeps the output of its oracle in `shim/N/reference/`:
//!
//! - `catalog.sha256` and `parameters.sha256`: one line for each group of the trace of the suite, as `<digest> <place> <messages> <rows> <query>`. The digest is the SHA-256 of the backend messages after the rules of spec/05 section 5.2 and the exclusions of section 5.3, one message on each line. The digest is `unstable` when the two runs of the oracle gave different answers.
//! - `suites.toml`: the result of the oracle for each suite of the version. The output of the regression tests, the isolation specs and the `libpq_pipeline` traces is the expected files of the pin, and `corpus/postgres/N/manifest.sha256` holds their digests.
//!
//! A full trace of the catalog suite is about 17 MB, so the repository keeps the digests and not the traces.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::compare::{Answer, Place, answers};
use crate::import::hex;
use crate::report::Record;
use crate::scram::sha256;
use crate::toml::{self, Table, Value};
use crate::trace::Line;

/// The suites with a digest file.
pub(crate) const DIGEST_SUITES: [&str; 2] = ["catalog", "parameters"];

/// The suites in `suites.toml`.
pub(crate) const SUITES: [&str; 5] = ["regress", "isolation", "pipeline", "catalog", "parameters"];

/// The word for a group that differs between the two runs of the oracle.
const UNSTABLE: &str = "unstable";

/// One line of a digest file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) place: Place,
    /// `None` when the group is unstable.
    pub(crate) digest: Option<String>,
    pub(crate) messages: usize,
    pub(crate) rows: usize,
    pub(crate) query: String,
}

fn digest(a: &Answer) -> String {
    let mut text = String::new();
    for m in &a.messages {
        text.push_str(m);
        text.push('\n');
    }
    hex(&sha256(text.as_bytes()))
}

/// The entries of the two oracle runs of a suite. A group with different answers in the two runs is unstable.
pub(crate) fn entries(first: &[Line], second: &[Line]) -> Result<Vec<Entry>, String> {
    let a = answers(first);
    let b = answers(second);
    if a.len() != b.len() {
        return Err(format!("the two oracle runs have {} and {} groups", a.len(), b.len()));
    }
    let mut out = Vec::new();
    for (x, y) in a.iter().zip(&b) {
        if x.place != y.place || x.query != y.query {
            return Err(format!("the two oracle runs differ in the queries at {:?}", x.place));
        }
        let stable = x.messages == y.messages;
        out.push(Entry {
            place: x.place,
            digest: stable.then(|| digest(x)),
            messages: x.messages.len(),
            rows: x.rows,
            query: x.query.clone(),
        });
    }
    Ok(out)
}

/// The query on one line: a backslash, a line feed, a carriage return and a tab get an escape.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// The text of a digest file. `header` gives the comment lines at the top.
pub(crate) fn write_entries(header: &[String], entries: &[Entry]) -> String {
    let mut out = String::new();
    for h in header {
        let _ = writeln!(out, "# {h}");
    }
    out.push_str("# digest, connection.group, messages, rows, query\n");
    for e in entries {
        let d = e.digest.as_deref().unwrap_or(UNSTABLE);
        let _ = writeln!(
            out,
            "{d} {}.{} {} {} {}",
            e.place.0,
            e.place.1,
            e.messages,
            e.rows,
            escape(&e.query)
        );
    }
    out
}

/// Reads a digest file.
pub(crate) fn read_entries(text: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let bad = || format!("line {}: {line:?} is not a digest line", i + 1);
        let mut f = line.splitn(5, ' ');
        let mut next = || f.next().ok_or_else(bad);
        let d = next()?;
        let place = next()?;
        let messages = next()?.parse().map_err(|_| bad())?;
        let rows = next()?.parse().map_err(|_| bad())?;
        let query = unescape(next().unwrap_or(""));
        let (conn, group) = place.split_once('.').ok_or_else(bad)?;
        let place = (conn.parse().map_err(|_| bad())?, group.parse().map_err(|_| bad())?);
        let digest = match d {
            UNSTABLE => None,
            d if d.len() == 64 && d.bytes().all(|b| b.is_ascii_hexdigit()) => Some(d.to_string()),
            _ => return Err(bad()),
        };
        out.push(Entry { place, digest, messages, rows, query });
    }
    Ok(out)
}

/// The result of a check of new entries against the reference.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Check {
    pub(crate) equal: usize,
    /// The groups that are unstable in the reference or in the new runs, so they are not compared.
    pub(crate) unstable: usize,
    /// The groups with a different digest or a different query, as `(place, query, reason)`.
    pub(crate) differ: Vec<(Place, String, String)>,
}

/// Compares new entries with the reference. A group that is unstable on either side is not compared, as in the unstable filter of spec/21 section 21.1.
pub(crate) fn check(reference: &[Entry], now: &[Entry]) -> Check {
    let mut out = Check::default();
    let now_by: BTreeMap<Place, &Entry> = now.iter().map(|e| (e.place, e)).collect();
    let reference_by: BTreeMap<Place, &Entry> = reference.iter().map(|e| (e.place, e)).collect();
    for r in reference {
        let Some(n) = now_by.get(&r.place) else {
            out.differ.push((r.place, r.query.clone(), "no group in the new runs".into()));
            continue;
        };
        if n.query != r.query {
            out.differ.push((r.place, r.query.clone(), format!("the query is now {:?}", n.query)));
            continue;
        }
        match (&r.digest, &n.digest) {
            (Some(a), Some(b)) if a == b => out.equal += 1,
            (Some(_), Some(_)) => out.differ.push((
                r.place,
                r.query.clone(),
                format!(
                    "{} messages and {} rows, now {} messages and {} rows",
                    r.messages, r.rows, n.messages, n.rows
                ),
            )),
            _ => out.unstable += 1,
        }
    }
    for n in now.iter().filter(|n| !reference_by.contains_key(&n.place)) {
        out.differ.push((n.place, n.query.clone(), "no group in the reference".into()));
    }
    out
}

/// The text of `suites.toml`: the result of each suite on the oracle. `cases` counts the unstable cases too, so it does not change between runs.
pub(crate) fn write_suites(version: u32, commit: &str, records: &[Record]) -> String {
    let mut out = format!(
        "# The result of each suite on the oracle of {version} (spec/03 section 3.15). Written by `rupg-compat reference`.\n\
         # The output of regress, isolation and pipeline is the expected files of the pin. Their SHA-256 digests are in corpus/postgres/{version}/manifest.sha256.\n\
         # The answers of catalog and parameters are in catalog.sha256 and parameters.sha256.\n\
         version = {version}\ncommit = \"{commit}\"\n"
    );
    for r in records {
        let _ = write!(
            out,
            "\n[{}]\nstarted = \"{}\"\ncases = {}\npassed = {}\nunstable = {}\n",
            r.suite,
            r.started,
            r.total + r.unstable.len(),
            r.passed,
            r.unstable.len()
        );
    }
    out
}

fn get(t: &Table, key: &str) -> Option<i64> {
    t.get(key).and_then(Value::as_int)
}

/// Compares two `suites.toml` texts. The start time and the count of unstable cases may change. The version, the commit, the count of cases and the count of failed cases may not.
pub(crate) fn check_suites(old: &str, now: &str) -> Result<Vec<String>, String> {
    let old = toml::parse(old).map_err(|e| e.to_string())?;
    let now = toml::parse(now).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for key in ["version", "commit"] {
        if old.get(key) != now.get(key) {
            out.push(format!("{key} is {:?}, now {:?}", old.get(key), now.get(key)));
        }
    }
    for suite in SUITES {
        let (Some(a), Some(b)) =
            (old.get(suite).and_then(Value::as_table), now.get(suite).and_then(Value::as_table))
        else {
            out.push(format!("[{suite}] is not in both files"));
            continue;
        };
        let failed = |t: &Table| Some(get(t, "cases")? - get(t, "unstable")? - get(t, "passed")?);
        if get(a, "cases") != get(b, "cases") {
            out.push(format!(
                "[{suite}] has {:?} cases, now {:?}",
                get(a, "cases"),
                get(b, "cases")
            ));
        }
        if failed(a) != failed(b) {
            out.push(format!("[{suite}] has {:?} failed cases, now {:?}", failed(a), failed(b)));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::Trace;

    fn lines(text: &str) -> Vec<Line> {
        Trace::parse(text).unwrap().lines
    }

    const START: &str = "1 F StartupMessage 3.0 \"user\" \"postgres\" \"database\" \"postgres\"\n\
        1 B ReadyForQuery I\n";

    fn run(a: &str, b: &str) -> String {
        format!(
            "{START}1 F Query \"SELECT a\"\n\
             1 B RowDescription [(\"a\" 0 0 23 4 -1 0)]\n\
             1 B DataRow [\"{a}\"]\n\
             1 B CommandComplete \"SELECT 1\"\n\
             1 B ReadyForQuery I\n\
             1 F Query \"SELECT b\"\n\
             1 B RowDescription [(\"b\" 0 0 25 -1 -1 0)]\n\
             1 B DataRow [\"{b}\"]\n\
             1 B CommandComplete \"SELECT 1\"\n\
             1 B ReadyForQuery I\n"
        )
    }

    #[test]
    fn a_group_that_differs_between_the_oracle_runs_is_unstable() {
        let e = entries(&lines(&run("1", "x")), &lines(&run("1", "y"))).unwrap();
        assert_eq!(e.len(), 3);
        assert_eq!(e[1].query, "SELECT a");
        assert_eq!((e[1].messages, e[1].rows), (4, 1));
        assert!(e[1].digest.is_some());
        assert_eq!(e[2].digest, None);
    }

    #[test]
    fn the_file_reads_back() {
        let mut e = entries(&lines(&run("1", "x")), &lines(&run("1", "y"))).unwrap();
        e[2].query = "SELECT 'a\\b'\n\tFROM t".into();
        let text = write_entries(&["PostgreSQL 18".into()], &e);
        assert!(text.starts_with("# PostgreSQL 18\n"));
        assert!(text.contains(" 1.1 4 1 SELECT a\n"));
        assert!(text.contains("unstable 1.2 4 1 SELECT 'a\\\\b'\\n\\tFROM t\n"));
        assert_eq!(read_entries(&text).unwrap(), e);
        assert!(read_entries("abc 1.0 1 1 x\n").is_err());
        assert!(read_entries("unstable 1 1 1 x\n").is_err());
    }

    #[test]
    fn the_check_finds_a_changed_answer() {
        let reference = entries(&lines(&run("1", "x")), &lines(&run("1", "y"))).unwrap();
        let same = entries(&lines(&run("1", "z")), &lines(&run("1", "z"))).unwrap();
        let c = check(&reference, &same);
        assert_eq!((c.equal, c.unstable, c.differ.len()), (2, 1, 0));
        let changed = entries(&lines(&run("2", "z")), &lines(&run("2", "z"))).unwrap();
        let c = check(&reference, &changed);
        assert_eq!((c.equal, c.unstable), (1, 1));
        assert_eq!(c.differ.len(), 1);
        assert_eq!(c.differ[0].1, "SELECT a");
        let c = check(&reference, &reference[..2]);
        assert_eq!(c.differ[0].2, "no group in the new runs");
    }

    fn record(suite: &str, total: usize, passed: usize, unstable: usize) -> Record {
        Record {
            suite: suite.into(),
            version: 18,
            commit: "abc".into(),
            server: "127.0.0.1:54318".into(),
            side: "oracle".into(),
            started: "2026-10-07T00:00:00Z".into(),
            seconds: 1,
            total,
            passed,
            unstable: (0..unstable).map(|i| i.to_string()).collect(),
            checked: None,
        }
    }

    #[test]
    fn the_suites_file_allows_other_unstable_cases() {
        let records = |cat: (usize, usize, usize)| {
            SUITES
                .iter()
                .map(|s| {
                    if *s == "catalog" {
                        record(s, cat.0, cat.1, cat.2)
                    } else {
                        record(s, 9, 9, 0)
                    }
                })
                .collect::<Vec<_>>()
        };
        let old = write_suites(18, "abc", &records((203, 203, 6)));
        assert!(old.contains("[catalog]\nstarted = \"2026-10-07T00:00:00Z\"\ncases = 209\npassed = 203\nunstable = 6\n"));
        assert_eq!(check_suites(&old, &old), Ok(vec![]));
        let now = write_suites(18, "abc", &records((201, 201, 8)));
        assert_eq!(check_suites(&old, &now), Ok(vec![]));
        let now = write_suites(18, "abc", &records((209, 208, 0)));
        assert_eq!(
            check_suites(&old, &now).unwrap(),
            ["[catalog] has Some(0) failed cases, now Some(1)"]
        );
        let now = write_suites(18, "def", &records((203, 203, 6)));
        assert_eq!(check_suites(&old, &now).unwrap().len(), 1);
    }
}
