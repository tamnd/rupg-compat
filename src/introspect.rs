//! The catalog suite, the parameter suite (spec/05 section 5.6.5) and the types suite.
//!
//! Each suite is a list of cases. Most cases are simple queries, and a case of the types suite can use the extended query protocol. The runner sends the list to the oracle two times and to the server under test one time, and keeps each answer as trace lines. Then it compares the answers with the rules of `compare`, as replay does: the groups that differ between the two oracle runs leave the comparison, and the excluded items and the rules of spec/05 section 5.2 apply.
//!
//! - The catalog suite runs `SELECT *` on each catalog, each system view and each `information_schema` view of the import, in a new database. With `--schema gate`, the new database first gets the gate schema of `clients/gate.sql`, and the suite is `catalog-gate`.
//! - The catalog corpus suite runs the catalog queries of the client traces in `corpus/traces` (spec/05 section 5.6.5), in a new database with the gate schema. A query is in the corpus when it starts with `SELECT` or `WITH`, reads the catalog and has no parameters.
//! - The parameter suite runs `SHOW ALL`, `SELECT * FROM pg_settings` and, for each parameter, `SET p TO DEFAULT`, `set_config` with the value of the oracle, and `SHOW p`.
//! - The types suite runs three cases for each value of `suites/types/values.txt` (spec/07 section 7.9): the text output of the value, its binary output from the send function, and the text output of the binary value of the oracle after the receive function of the server.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::client::{self, Conn, Target};
use crate::compare::{self, Outcome};
use crate::import;
use crate::message::{Dir, Msg, Val};
use crate::replay;
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
    /// The messages that the case sends, up to `Sync`. When the list is empty, the case sends `sql` as a simple query.
    pub(crate) front: Vec<Msg>,
}

impl Case {
    /// A case that sends `sql` as a simple query.
    pub(crate) fn query(name: impl Into<String>, sql: impl Into<String>) -> Case {
        Case { name: name.into(), sql: sql.into(), front: Vec::new() }
    }
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
                cases.push(Case::query(
                    format!("{schema}.{name}"),
                    format!("SELECT * FROM {schema}.{name}"),
                ));
            }
        }
    }
    Ok(cases)
}

/// The words of a query of the catalog corpus that read the catalog, apart from the words that start with `pg_`.
const CATALOG_WORDS: [&str; 14] = [
    "information_schema",
    "regclass",
    "regtype",
    "regproc",
    "regprocedure",
    "regnamespace",
    "regrole",
    "regoper",
    "format_type",
    "version",
    "current_schema",
    "current_schemas",
    "obj_description",
    "col_description",
];

/// The words that make a query of a trace leave the catalog corpus: the words that change data, and the starts of the names of functions that change a state or take a lock.
const WRITE_WORDS: [&str; 5] = ["into", "insert", "update", "delete", "merge"];
const WRITE_FUNCTIONS: [&str; 13] = [
    "pg_sleep",
    "pg_advisory",
    "pg_try_advisory",
    "pg_cancel_backend",
    "pg_terminate_backend",
    "pg_notify",
    "pg_reload_conf",
    "pg_stat_reset",
    "pg_current_xact_id",
    "txid_current",
    "set_config",
    "nextval",
    "setval",
];

/// Whether `sql` goes in the catalog corpus: it starts with `SELECT` or `WITH`, it reads the catalog, it changes nothing, and it has no parameters. A word in a string or a comment counts too.
fn corpus_query(sql: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$'))
        .filter(|w| !w.is_empty())
        .collect();
    let parameter =
        |w: &&str| w.split('$').skip(1).any(|n| n.starts_with(|c: char| c.is_ascii_digit()));
    matches!(words.first(), Some(&("select" | "with")))
        && words.iter().any(|w| w.starts_with("pg_") || CATALOG_WORDS.contains(w))
        && !words.iter().any(|w| WRITE_WORDS.contains(w))
        && !words.iter().any(|w| WRITE_FUNCTIONS.iter().any(|f| w.starts_with(f)))
        && !words.iter().any(parameter)
        && !lower.contains("lo_")
}

/// The catalog corpus suite: each query of the traces in `dir` that [`corpus_query`] accepts, one time. The name of a case is the name of the first trace that sends the query and the number of the query in that trace.
pub(crate) fn corpus_cases(dir: &Path) -> Result<Vec<Case>, String> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "trace"))
        .collect();
    files.sort();
    let mut seen = BTreeSet::new();
    let mut cases = Vec::new();
    for path in files {
        let trace = Trace::read(&path)?;
        let client = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mut n = 0;
        for msg in trace.lines.iter().filter(|l| l.dir == Dir::F).filter_map(Line::msg) {
            let sql = match msg.name() {
                "Query" => msg.vals.first(),
                "Parse" => msg.vals.get(1),
                _ => None,
            };
            let Some(sql) = sql.and_then(Val::bytes) else { continue };
            let sql = String::from_utf8_lossy(sql).trim().trim_end_matches(';').to_string();
            if corpus_query(&sql) && seen.insert(sql.clone()) {
                n += 1;
                cases.push(Case::query(format!("{client} {n}"), sql));
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
        Case::query("SHOW ALL", "SHOW ALL"),
        Case::query("pg_settings", "SELECT * FROM pg_catalog.pg_settings"),
    ];
    for (name, value) in values {
        cases.push(Case::query(format!("{name} default"), format!("SET {name} TO DEFAULT")));
        if let Some(v) = value {
            cases.push(Case::query(
                format!("{name} set"),
                format!("SELECT pg_catalog.set_config({}, {}, false)", literal(name), literal(v)),
            ));
        }
        cases.push(Case::query(format!("{name} show"), format!("SHOW {name}")));
    }
    cases
}

/// The values of the types suite: the pairs of type and value of `values.txt` in `dir`. A line that starts with `#` is a comment. The value is the rest of the line after the first space, so it can start with spaces. A line with only a type has the empty value.
pub(crate) fn type_values(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let path = dir.join("values.txt");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (ty, value) = line.split_once(' ').unwrap_or((line, ""));
        if ty.is_empty() {
            return Err(format!("{}:{}: no type", path.display(), n + 1));
        }
        out.push((ty.to_string(), value.to_string()));
    }
    Ok(out)
}

/// The messages of one run of `sql` with the extended query protocol, with the formats of the parameters and of the result. `Describe` adds the `RowDescription`, so the types and the formats of the columns are compared too.
fn extended(sql: &str, types: &[u32], formats: &[i64], params: Vec<Val>, result: i64) -> Vec<Msg> {
    let ints = |v: &[i64]| Val::List(v.iter().map(|&i| Val::Int(i)).collect());
    let types = types.iter().map(|&t| Val::Int(i64::from(t))).collect();
    vec![
        Msg::new(
            Dir::F,
            "Parse",
            vec![client::str_val(""), client::str_val(sql), Val::List(types)],
        ),
        Msg::new(
            Dir::F,
            "Bind",
            vec![
                client::str_val(""),
                client::str_val(""),
                ints(formats),
                Val::List(params),
                ints(&[result]),
            ],
        ),
        Msg::new(Dir::F, "Describe", vec![Val::Byte(b'P'), client::str_val("")]),
        Msg::new(Dir::F, "Execute", vec![client::str_val(""), Val::Int(0)]),
        Msg::new(Dir::F, "Sync", vec![]),
    ]
}

/// The type OID of a value of the types suite and the binary value that the oracle gives for it. The binary value is None when the oracle rejects the value.
pub(crate) type Binary = (u32, Option<Vec<u8>>);

/// The type OID and the binary value of the oracle for each value of the types suite.
pub(crate) fn binary_values(
    oracle: &Target,
    list: &[(String, String)],
) -> Result<Vec<Binary>, String> {
    let mut c = Conn::connect(oracle, "postgres")?;
    let mut out = Vec::new();
    for (ty, value) in list {
        let sql = format!("SELECT {}::regtype::oid", literal(ty));
        let oid = values(&c.query(&sql)?)
            .first()
            .and_then(|r| r.first().cloned().flatten())
            .and_then(|o| o.parse().ok())
            .ok_or_else(|| format!("{sql}: the oracle does not know the type"))?;
        for m in extended(&format!("SELECT {}::{ty}", literal(value)), &[], &[], vec![], 1) {
            c.send(&m)?;
        }
        let mut bytes = None;
        loop {
            let m = c.recv()?;
            match m.name() {
                "DataRow" => {
                    bytes = m.vals[0].list().first().and_then(Val::bytes).map(<[u8]>::to_vec)
                }
                "ReadyForQuery" => break,
                _ => {}
            }
        }
        out.push((oid, bytes));
    }
    Ok(out)
}

/// The types suite. For each value it has the case `text`, a simple query, and the case `send`, the same query with a binary result. When the oracle gave a binary value, the case `receive` binds that value as a binary parameter of the type and reads it back as text.
pub(crate) fn type_cases(list: &[(String, String)], binary: &[Binary]) -> Vec<Case> {
    let mut cases = Vec::new();
    for ((ty, value), (oid, bytes)) in list.iter().zip(binary) {
        let name = format!("{ty} {value:?}");
        let sql = format!("SELECT {}::{ty}", literal(value));
        cases.push(Case::query(format!("{name} text"), sql.clone()));
        let front = extended(&sql, &[], &[], vec![], 1);
        cases.push(Case { name: format!("{name} send"), sql, front });
        if let Some(b) = bytes {
            let sql = "SELECT $1".to_string();
            let front = extended(&sql, &[*oid], &[1], vec![Val::Str(b.clone())], 0);
            cases.push(Case { name: format!("{name} receive"), sql, front });
        }
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
pub(crate) fn values(msgs: &[Msg]) -> Vec<Vec<Option<String>>> {
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
        let query = [Msg::new(Dir::F, "Query", vec![client::str_val(&case.sql)])];
        let front = if case.front.is_empty() { &query[..] } else { &case.front[..] };
        for m in front {
            c.send(m)?;
            lines.push(Line { conn: CONN, dir: Dir::F, event: Event::Msg(m.clone()) });
        }
        loop {
            let m = c.recv()?;
            let done = m.name() == "ReadyForQuery";
            lines.push(Line { conn: CONN, dir: Dir::B, event: Event::Msg(m) });
            if done {
                break;
            }
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

/// Makes the database new and runs the script `setup` in it, when the script is not empty.
fn reset(target: &Target, database: &str, setup: &str) -> Result<(), String> {
    client::reset_database(target, database)?;
    if !setup.is_empty() {
        Conn::connect(target, database)?.execute(setup)?;
    }
    Ok(())
}

/// The first OID of the system collations that `initdb` made on `target`, if it made any. The objects that `initdb` makes after them get OIDs that depend on the locales of the host (spec/05 section 5.3 item 16).
fn first_collation(target: &Target, database: &str) -> Result<Option<u32>, String> {
    let sql = format!(
        "SELECT min(oid) FROM pg_catalog.pg_collation WHERE oid >= {} AND oid < {}",
        replay::FIRST_UNPINNED_OID,
        replay::FIRST_NORMAL_OID
    );
    let rows = Conn::connect(target, database)?.query(&sql)?;
    if let Some(e) = rows.iter().find(|m| m.name() == "ErrorResponse") {
        return Err(format!("{sql}: {}", client::error_text(e)));
    }
    Ok(rows
        .iter()
        .find(|m| m.name() == "DataRow")
        .and_then(|m| m.vals[0].list().first()?.bytes())
        .and_then(|b| std::str::from_utf8(b).ok()?.parse().ok()))
}

/// Runs a suite: two oracle runs, the run under test, and the comparison. Before the runs, the database gets the objects of `setup`. It writes the three traces to `out`.
pub(crate) fn run_suite(
    suite: &str,
    cases: &[Case],
    sides: &Sides,
    database: &str,
    setup: &str,
    out: &Path,
    header: &[String],
) -> Result<(Run, Outcome), String> {
    let started = Instant::now();
    let fresh = database != "postgres";
    if fresh {
        reset(&sides.oracle, database, setup)?;
    }
    let first = run_cases(&sides.oracle, database, cases)?;
    let second = run_cases(&sides.oracle, database, cases)?;
    let got = match &sides.server {
        Some(server) => {
            // The oracle has the work database of the harness, so the server gets the same databases before a query of `pg_database`. The reset comes after, so the order of the OIDs is the same as on the oracle.
            client::same_databases(&sides.oracle, server)?;
            if fresh {
                reset(server, database, setup)?;
            }
            Some(run_cases(server, database, cases)?)
        }
        None => None,
    };
    let mut collations = first_collation(&sides.oracle, database)?;
    if let Some(server) = &sides.server
        && let Some(oid) = first_collation(server, database)?
    {
        collations = Some(collations.map_or(oid, |o| o.min(oid)));
    }
    let unstable = compare::unstable(&first, &second);
    let outcome =
        compare::compare_with(&first, got.as_ref().unwrap_or(&first), &unstable, collations);
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
    fn the_catalog_corpus_takes_the_catalog_queries_without_parameters() {
        assert!(corpus_query("SELECT oid, typname FROM pg_catalog.pg_type"));
        assert!(corpus_query("with t as (select 1) select * from information_schema.tables, t"));
        assert!(corpus_query("SELECT 'x'::regclass"));
        assert!(!corpus_query("SELECT * FROM pg_type WHERE oid = $1"));
        assert!(!corpus_query("SELECT * FROM compat_items"));
        assert!(!corpus_query("INSERT INTO t SELECT * FROM pg_class"));
        assert!(!corpus_query("SELECT relname INTO t FROM pg_class"));
        assert!(!corpus_query("SELECT pg_advisory_lock(1), pg_backend_pid()"));
        assert!(!corpus_query("SELECT nextval('s'), version()"));
        // A dollar quote is not a parameter.
        assert!(corpus_query("SELECT $$a$$, pg_backend_pid()"));
    }

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
    fn each_type_value_has_three_cases() {
        let dir = std::env::temp_dir().join(format!("rupg-compat-types-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("values.txt"), "# comment\nint4 1\nbool  off\nint4 x\ntext\n").unwrap();
        let list = type_values(&dir).unwrap();
        fs::write(dir.join("values.txt"), " 1\n").unwrap();
        assert!(type_values(&dir).unwrap_err().ends_with(":1: no type"));
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(list[1], ("bool".to_string(), " off".to_string()));
        assert_eq!(list[3], ("text".to_string(), String::new()));
        let binary = [(23, Some(vec![0, 0, 0, 1])), (16, Some(vec![0])), (23, None)];
        let cases = type_cases(&list, &binary);
        let names: Vec<&str> = cases.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "int4 \"1\" text",
                "int4 \"1\" send",
                "int4 \"1\" receive",
                "bool \" off\" text",
                "bool \" off\" send",
                "bool \" off\" receive",
                "int4 \"x\" text",
                "int4 \"x\" send"
            ]
        );
        assert_eq!(cases[0].sql, "SELECT '1'::int4");
        assert!(cases[0].front.is_empty());
        let front: Vec<&str> = cases[1].front.iter().map(Msg::name).collect();
        assert_eq!(front, ["Parse", "Bind", "Describe", "Execute", "Sync"]);
        assert_eq!(cases[1].front[1].vals[4], Val::List(vec![Val::Int(1)]));
        let receive = &cases[2].front;
        assert_eq!(receive[0].vals[2], Val::List(vec![Val::Int(23)]));
        assert_eq!(receive[1].vals[2], Val::List(vec![Val::Int(1)]));
        assert_eq!(receive[1].vals[3], Val::List(vec![Val::Str(vec![0, 0, 0, 1])]));
        assert_eq!(receive[1].vals[4], Val::List(vec![Val::Int(0)]));
    }

    #[test]
    fn unstable_cases_leave_the_pass_set() {
        let cases = vec![
            Case::query("a", "SELECT 1"),
            Case::query("b", "SELECT 2"),
            Case::query("c", "SELECT 3"),
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
