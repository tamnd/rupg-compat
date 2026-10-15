//! The lists of the known failures of a client (spec/21 section 21.4.2).
//!
//! `clients/<name>/oracle-fail.txt` gives the tests that fail against the oracle. They are not counted. `clients/<name>/expected-fail.txt` gives the tests that pass against the oracle and fail against rupg, each as `<test>: <cause>, <milestone>`. The second file may only get shorter: a test that starts to pass is removed from it in the same change. In both files, an empty line or a line that starts with `#` is not a test.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The two lists of one client.
#[derive(Debug, Default)]
pub(crate) struct Fails {
    /// The tests that fail against the oracle.
    pub(crate) oracle: Vec<String>,
    /// The tests that fail against rupg, with the cause and the milestone of each.
    pub(crate) expected: BTreeMap<String, String>,
}

/// The lines of a list that are tests.
fn tests(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(n, l)| (n + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
}

/// True when the reason of an `expected-fail.txt` line ends with a milestone, `M` and a number.
fn has_milestone(reason: &str) -> bool {
    let last = reason.rsplit([' ', ',']).next().unwrap_or_default();
    last.len() > 1 && last.starts_with('M') && last[1..].bytes().all(|b| b.is_ascii_digit())
}

/// Reads the two lists of a client from its text.
pub(crate) fn parse(oracle: &str, expected: &str) -> Result<Fails, String> {
    let mut fails =
        Fails { oracle: tests(oracle).map(|(_, l)| l.to_string()).collect(), ..Fails::default() };
    for (n, line) in tests(expected) {
        let bad = || {
            format!("expected-fail.txt line {n}: give the test as `<test>: <cause>, <milestone>`")
        };
        let (test, reason) = line.split_once(':').ok_or_else(bad)?;
        let (test, reason) = (test.trim(), reason.trim());
        if test.is_empty() || !has_milestone(reason) {
            return Err(bad());
        }
        if fails.oracle.iter().any(|t| t == test) {
            return Err(format!("expected-fail.txt line {n}: {test} is also in oracle-fail.txt"));
        }
        if fails.expected.insert(test.to_string(), reason.to_string()).is_some() {
            return Err(format!("expected-fail.txt line {n}: {test} is in the list two times"));
        }
    }
    Ok(fails)
}

/// Reads the two lists of the client in `clients/<name>`.
pub(crate) fn load(name: &str) -> Result<Fails, String> {
    let dir = Path::new("clients").join(name);
    let read = |file: &str| {
        let path = dir.join(file);
        fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
    };
    parse(&read("oracle-fail.txt")?, &read("expected-fail.txt")?)
        .map_err(|e| format!("clients/{name}/{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lists_give_the_tests_and_the_causes() {
        let fails = parse("# a comment\n\nscenario\n", "gate: pg_get_partkeydef, M2\n").unwrap();
        assert_eq!(fails.oracle, ["scenario"]);
        assert_eq!(fails.expected["gate"], "pg_get_partkeydef, M2");
    }

    #[test]
    fn a_line_with_no_milestone_is_an_error() {
        let message = parse("", "gate: pg_get_partkeydef\n").unwrap_err();
        assert!(message.starts_with("expected-fail.txt line 1:"), "{message}");
        assert!(parse("", "gate\n").is_err());
        assert!(parse("", ": no test, M2\n").is_err());
        assert!(parse("", "gate: a, M2\ngate: b, M3\n").is_err());
        assert!(parse("gate\n", "gate: a, M2\n").is_err());
    }
}
