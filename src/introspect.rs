//! The catalog suite and the parameter suite (spec/05 section 5.6.5).
//!
//! Each suite is a list of simple queries. The runner sends the list to the oracle two times and to the server under test one time, and keeps each answer as trace lines. Then it compares the answers with the rules of `compare`, as replay does: the groups that differ between the two oracle runs leave the comparison, and the 15 excluded items and the rules of spec/05 section 5.2 apply.
//!
//! - The catalog suite runs `SELECT *` on each catalog, each system view and each `information_schema` view of the import, in a new database.
//! - The parameter suite runs `SHOW ALL`, `SELECT * FROM pg_settings` and, for each parameter, `SET p TO DEFAULT`, `set_config` with the value of the oracle, and `SHOW p`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::client::{self, Conn, Target};
use crate::compare::{self, Outcome};
use crate::import;
use crate::message::{Dir, Msg, Val};
use crate::suites::{Checked, Run};
use crate::trace::{Event, Line, Trace};

/// The database that the catalog suite makes new on each server.
pub(crate) const CATALOG_DATABASE: &str = "compat_catalog";

/// The connection number of the suite in the trace lines.
const CONN: u32 = 1;

/// One query of a suite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Case {
    pub(crate) name: String,
    pub(crate) sql: String,
}

/// The catalog suite: `SELECT *` on the relations of the lists of the import.
pub(crate) fn catalog_cases(dir: &Path) -> Result<Vec<Case>, String> {
    let mut cases = Vec::new();
    for (file, schema) in [
        ("catalogs.txt", "pg_catalog"),
        ("system_views.txt", "pg_catalog"),
        ("information_schema.txt", "information_schema"),
    ] {
        for words in import::list(dir, file)? {
            if let Some(name) = words.first() {
                cases.push(Case {
                    name: format!("{schema}.{name}"),
                    sql: format!("SELECT * FROM {schema}.{name}"),
                });
            }
        }
    }
    Ok(cases)
}

/// A string literal of SQL.
fn literal(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// The parameter suite. `values` has the value that `SHOW` gives on the oracle for each parameter, or None when `SHOW` fails.
pub(crate) fn parameter_cases(values: &[(String, Option<String>)]) -> Vec<Case> {
    let mut cases = vec![
        Case { name: "SHOW ALL".into(), sql: "SHOW ALL".into() },
        Case { name: "pg_settings".into(), sql: "SELECT * FROM pg_catalog.pg_settings".into() },
    ];
    for (name, value) in values {
        cases.push(Case { name: format!("{name} default"), sql: format!("SET {name} TO DEFAULT") });
        if let Some(v) = value {
            cases.push(Case {
                name: format!("{name} set"),
                sql: format!(
                    "SELECT pg_catalog.set_config({}, {}, false)",
                    literal(name),
                    literal(v)
                ),
            });
        }
        cases.push(Case { name: format!("{name} show"), sql: format!("SHOW {name}") });
    }
    cases
}

/// The parameters of the suite: `parameters.txt` of the import when it has one, else the rows of `pg_settings` on the oracle. Only the import of 19 has `parameters.txt`, because `guc_parameters.dat` is new in 19.
pub(crate) fn parameter_names(
    dir: &Path,
    oracle: &Target,
) -> Result<(Vec<String>, String), String> {
    if dir.join("parameters.txt").exists() {
        let names =
            import::list(dir, "parameters.txt")?.into_iter().filter_map(|w| w.into_iter().next());
        return Ok((names.collect(), "parameters.txt".into()));
    }
    let mut c = Conn::connect(oracle, "postgres")?;
    let rows = values(&c.query("SELECT name FROM pg_catalog.pg_settings ORDER BY name")?);
    Ok((
        rows.into_iter().filter_map(|r| r.into_iter().next().flatten()).collect(),
        "pg_settings of the oracle".into(),
    ))
}

/// The text values of the `DataRow` messages of an answer.
fn values(msgs: &[Msg]) -> Vec<Vec<Option<String>>> {
    msgs.iter()
        .filter(|m| m.name() == "DataRow")
        .map(|m| {
            m.vals[0]
                .list()
                .iter()
                .map(|v| v.bytes().map(|b| String::from_utf8_lossy(b).into_owned()))
                .collect()
        })
        .collect()
}

/// The value that `SHOW` gives for each parameter on the oracle.
pub(crate) fn show_values(
    oracle: &Target,
    names: &[String],
) -> Result<Vec<(String, Option<String>)>, String> {
    let mut c = Conn::connect(oracle, "postgres")?;
    let mut out = Vec::new();
    for name in names {
        let msgs = c.query(&format!("SHOW {name}"))?;
        let v = values(&msgs).into_iter().next().and_then(|r| r.into_iter().next().flatten());
        out.push((name.clone(), v));
    }
    Ok(out)
}

/// Runs the cases on one connection and returns the trace lines: each `Query` and the backend messages up to `ReadyForQuery`.
pub(crate) fn run_cases(
    target: &Target,
    database: &str,
    cases: &[Case],
) -> Result<Vec<Line>, String> {
    let mut c = Conn::connect(target, database)?;
    let mut lines = Vec::new();
    for case in cases {
        let q = Msg::new(Dir::F, "Query", vec![Val::Str(case.sql.clone().into_bytes())]);
        lines.push(Line { conn: CONN, dir: Dir::F, event: Event::Msg(q) });
        for m in c.query(&case.sql)? {
            lines.push(Line { conn: CONN, dir: Dir::B, event: Event::Msg(m) });
        }
    }
    Ok(lines)
}

/// The two sides of a run.
#[derive(Clone, Debug)]
pub(crate) struct Sides {
    pub(crate) oracle: Target,
    /// The server under test. Without it, the first oracle run is the answer under test.
    pub(crate) server: Option<Target>,
}

/// The pass set of the cases after the comparison. A case that the unstable filter took out is not in the pass set and not in the fail set.
pub(crate) fn verdicts(
    suite: &str,
    cases: &[Case],
    outcome: &Outcome,
    unstable: &BTreeSet<compare::Place>,
) -> Run {
    let mut run = Run { suite: suite.into(), ..Run::default() };
    let mut shares = Vec::new();
    for (i, case) in cases.iter().enumerate() {
        if unstable.contains(&(CONN, i)) {
            run.unstable.push(case.name.clone());
        } else {
            run.tests.push((case.name.clone(), !outcome.diffs.contains_key(&(CONN, i))));
            shares.push((
                case.name.clone(),
                outcome.shares.get(&(CONN, i)).copied().unwrap_or_default(),
            ));
        }
    }
    run.checked = Some(Checked::new(outcome, shares));
    run
}

/// Runs a suite: two oracle runs, the run under test, and the comparison. It writes the three traces to `out`.
pub(crate) fn run_suite(
    suite: &str,
    cases: &[Case],
    sides: &Sides,
    database: &str,
    out: &Path,
    header: &[String],
) -> Result<(Run, Outcome), String> {
    let started = Instant::now();
    let fresh = database != "postgres";
    if fresh {
        client::reset_database(&sides.oracle, database)?;
    }
    let first = run_cases(&sides.oracle, database, cases)?;
    let second = run_cases(&sides.oracle, database, cases)?;
    let got = match &sides.server {
        Some(server) => {
            if fresh {
                client::reset_database(server, database)?;
            }
            Some(run_cases(server, database, cases)?)
        }
        None => None,
    };
    let unstable = compare::unstable(&first, &second);
    let outcome = compare::compare(&first, got.as_ref().unwrap_or(&first), &unstable);
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let write = |name: &str, lines: &[Line]| {
        let t = Trace { header: header.to_vec(), lines: lines.to_vec() };
        let path = out.join(name);
        fs::write(&path, t.to_text()).map_err(|e| format!("{}: {e}", path.display()))
    };
    write("oracle-1.trace", &first)?;
    write("oracle-2.trace", &second)?;
    if let Some(g) = &got {
        write("server.trace", g)?;
    }
    let mut run = verdicts(suite, cases, &outcome, &unstable);
    run.seconds = started.elapsed().as_secs();
    Ok((run, outcome))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_cases_come_from_the_lists() {
        let dir =
            std::env::temp_dir().join(format!("rupg-compat-introspect-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("catalogs.txt"), "pg_class local\npg_database shared\n").unwrap();
        fs::write(dir.join("system_views.txt"), "pg_roles\n").unwrap();
        fs::write(dir.join("information_schema.txt"), "tables\n").unwrap();
        let cases = catalog_cases(&dir).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        let sql: Vec<&str> = cases.iter().map(|c| c.sql.as_str()).collect();
        assert_eq!(
            sql,
            [
                "SELECT * FROM pg_catalog.pg_class",
                "SELECT * FROM pg_catalog.pg_database",
                "SELECT * FROM pg_catalog.pg_roles",
                "SELECT * FROM information_schema.tables"
            ]
        );
    }

    #[test]
    fn each_parameter_has_three_cases() {
        let cases = parameter_cases(&[
            ("search_path".into(), Some("\"$user\", public".into())),
            ("lc_time".into(), Some("C'".into())),
            ("seed".into(), None),
        ]);
        let sql: Vec<&str> = cases.iter().map(|c| c.sql.as_str()).collect();
        assert_eq!(
            sql,
            [
                "SHOW ALL",
                "SELECT * FROM pg_catalog.pg_settings",
                "SET search_path TO DEFAULT",
                "SELECT pg_catalog.set_config('search_path', '\"$user\", public', false)",
                "SHOW search_path",
                "SET lc_time TO DEFAULT",
                "SELECT pg_catalog.set_config('lc_time', 'C''', false)",
                "SHOW lc_time",
                "SET seed TO DEFAULT",
                "SHOW seed"
            ]
        );
    }

    #[test]
    fn unstable_cases_leave_the_pass_set() {
        let cases = vec![
            Case { name: "a".into(), sql: "SELECT 1".into() },
            Case { name: "b".into(), sql: "SELECT 2".into() },
            Case { name: "c".into(), sql: "SELECT 3".into() },
        ];
        let mut outcome = Outcome::default();
        outcome.diffs.insert((CONN, 2), "want x".into());
        let run = verdicts("catalog", &cases, &outcome, &BTreeSet::from([(CONN, 1)]));
        assert_eq!(run.tests, [("a".to_string(), true), ("c".into(), false)]);
        assert_eq!(run.unstable, ["b"]);
        let checked = run.checked.unwrap();
        let names: Vec<&str> = checked.cases.iter().map(|c| c.0.as_str()).collect();
        assert_eq!(names, ["a", "c"]);
    }
}
