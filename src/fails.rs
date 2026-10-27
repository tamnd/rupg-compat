//! The lists of the known failures of a client or a suite (spec/21 section 21.4.2).
//!
//! The lists of a client are in `clients/<name>`, and the lists of a suite are in `suites/<name>`. `oracle-fail.txt` gives the tests that fail against the oracle. They are not counted. `expected-fail.txt` gives the tests that pass against the oracle and fail against rupg, each as `<test>: <cause>, <milestone>`. The second file may only get shorter: a test that starts to pass is removed from it in the same change. In both files, an empty line or a line that starts with `#` is not a test.

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
    load_dir(&Path::new("clients").join(name))
}

/// Reads the two lists in a directory.
pub(crate) fn load_dir(dir: &Path) -> Result<Fails, String> {
    let read = |file: &str| {
        let path = dir.join(file);
        fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
    };
    parse(&read("oracle-fail.txt")?, &read("expected-fail.txt")?)
        .map_err(|e| format!("{}/{e}", dir.display()))
}

/// Reads the excluded tests of a suite from its text: one test on each line as `<test>: <reason>`. The reason gives the item of spec/05 section 5.3 or the section of spec/16 that excludes the test.
pub(crate) fn parse_excluded(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (n, line) in tests(text) {
        let bad = || format!("excluded.txt line {n}: give the test as `<test>: <reason>`");
        let (test, reason) = line.split_once(':').ok_or_else(bad)?;
        let (test, reason) = (test.trim(), reason.trim());
        if test.is_empty() || reason.is_empty() {
            return Err(bad());
        }
        if out.iter().any(|(t, _)| t == test) {
            return Err(format!("excluded.txt line {n}: {test} is in the list two times"));
        }
        out.push((test.to_string(), reason.to_string()));
    }
    Ok(out)
}

/// Reads `excluded.txt` in a directory. A directory with no such file excludes no test.
pub(crate) fn load_excluded(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let path = dir.join("excluded.txt");
    match fs::read_to_string(&path) {
        Ok(text) => parse_excluded(&text).map_err(|e| format!("{}/{e}", dir.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Reads the milestone of each test of a suite from its text: one test on each line as `<test>: M<n>` or `<test>: M<n>, <reason>`.
pub(crate) fn parse_milestones(text: &str) -> Result<Vec<(String, u32)>, String> {
    let mut out: Vec<(String, u32)> = Vec::new();
    for (n, line) in tests(text) {
        let bad = || format!("milestones.txt line {n}: give the test as `<test>: M<n>, <reason>`");
        let (test, rest) = line.split_once(':').ok_or_else(bad)?;
        let test = test.trim();
        let milestone = rest.split(',').next().unwrap_or_default().trim();
        let number = milestone.strip_prefix('M').and_then(|m| m.parse().ok()).ok_or_else(bad)?;
        if test.is_empty() {
            return Err(bad());
        }
        if out.iter().any(|(t, _)| t == test) {
            return Err(format!("milestones.txt line {n}: {test} is in the list two times"));
        }
        out.push((test.to_string(), number));
    }
    Ok(out)
}

/// Reads `milestones.txt` in a directory. A directory with no such file gives no milestone.
pub(crate) fn load_milestones(dir: &Path) -> Result<Vec<(String, u32)>, String> {
    let path = dir.join("milestones.txt");
    match fs::read_to_string(&path) {
        Ok(text) => parse_milestones(&text).map_err(|e| format!("{}/{e}", dir.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// True when the results of a run agree with the lists: each test that fails is in `oracle-fail.txt` or in `expected-fail.txt`, and no test in `expected-fail.txt` passes. A line goes to the output for each test in `expected-fail.txt`, and the list in `dir` is named for a test to remove.
pub(crate) fn agree(tests: &[(String, bool)], fails: &Fails, dir: &str) -> bool {
    let mut good = true;
    for (test, passed) in tests {
        if fails.oracle.contains(test) {
            continue;
        }
        match (passed, fails.expected.get(test)) {
            (true, Some(_)) => {
                out!("  {test}: passes, so remove it from {dir}/expected-fail.txt");
                good = false;
            }
            (false, Some(cause)) => out!("  {test}: fails as expected-fail.txt gives: {cause}"),
            (false, None) => good = false,
            (true, None) => {}
        }
    }
    good
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

    #[test]
    fn a_run_agrees_when_each_failure_is_in_a_list() {
        let fails = parse("flaky\n", "insert: INSERT, M3\n").unwrap();
        let run = |results: &[(&str, bool)]| -> Vec<(String, bool)> {
            results.iter().map(|(t, p)| (t.to_string(), *p)).collect()
        };
        assert!(agree(&run(&[("a", true), ("insert", false), ("flaky", false)]), &fails, "d"));
        assert!(!agree(&run(&[("a", false), ("insert", false)]), &fails, "d"));
        assert!(!agree(&run(&[("insert", true)]), &fails, "d"));
    }

    #[test]
    fn the_pipeline_suite_has_its_lists() {
        load_dir(Path::new("suites/pipeline")).unwrap();
    }

    #[test]
    fn each_excluded_test_has_a_reason() {
        let list = parse_excluded("# a comment\n\nexplain: the plan, item 1\n").unwrap();
        assert_eq!(list, [("explain".to_string(), "the plan, item 1".to_string())]);
        assert!(parse_excluded("explain\n").is_err());
        assert!(parse_excluded("explain:\n").is_err());
        assert!(parse_excluded("a: x\na: y\n").is_err());
        let regress = load_excluded(Path::new("suites/regress")).unwrap();
        assert!(regress.iter().any(|(t, _)| t == "explain"));
        assert_eq!(load_excluded(Path::new("suites/types")).unwrap(), []);
    }

    #[test]
    fn each_test_has_a_milestone() {
        let list = parse_milestones("# a comment\nint4: M3\nplpgsql: M7, PL/pgSQL\n").unwrap();
        assert_eq!(list, [("int4".to_string(), 3), ("plpgsql".to_string(), 7)]);
        assert!(parse_milestones("int4\n").is_err());
        assert!(parse_milestones("int4: 3\n").is_err());
        assert!(parse_milestones("int4: M3\nint4: M5\n").is_err());
        let regress = load_milestones(Path::new("suites/regress")).unwrap();
        let excluded = load_excluded(Path::new("suites/regress")).unwrap();
        assert!(regress.iter().any(|(t, m)| t == "boolean" && *m == 3));
        assert!(regress.iter().all(|(t, _)| !excluded.iter().any(|(e, _)| e == t)));
    }
}
