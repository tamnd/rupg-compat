//! The report and the ratchet (spec/21 section 21.15).
//!
//! `rupg-compat report` reads the result files `results/N/*.toml` of the work directory and writes `reports/<date>/report.md`, with one table for each level and each version. Then it checks each row of `ratchet.toml`: the report fails when a number of the server under test is below its ratchet.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compare::{self, Count, Share};
use crate::pins::Pins;
use crate::suites::{Checked, quote};
use crate::toml::{self, Table, Value};

/// One result file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Record {
    pub(crate) suite: String,
    pub(crate) version: u32,
    pub(crate) commit: String,
    pub(crate) server: String,
    /// `oracle` or `server`.
    pub(crate) side: String,
    pub(crate) started: String,
    pub(crate) seconds: u64,
    pub(crate) total: usize,
    pub(crate) passed: usize,
    /// The cases that the unstable filter took out.
    pub(crate) unstable: Vec<String>,
    pub(crate) checked: Option<Checked>,
}

fn int(t: &Table, key: &str) -> Result<i64, String> {
    t.get(key).and_then(Value::as_int).ok_or(format!("no integer {key}"))
}

fn text(t: &Table, key: &str) -> Result<String, String> {
    t.get(key).and_then(Value::as_str).map(str::to_string).ok_or(format!("no string {key}"))
}

fn ints(v: Option<&Value>) -> Vec<usize> {
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_int).map(|n| n as usize).collect(),
        _ => Vec::new(),
    }
}

fn counts<const N: usize>(t: Option<&Table>) -> [Count; N] {
    let applied = ints(t.and_then(|t| t.get("applied")));
    let differed = ints(t.and_then(|t| t.get("differed")));
    std::array::from_fn(|i| Count {
        applied: applied.get(i).copied().unwrap_or(0),
        differed: differed.get(i).copied().unwrap_or(0),
    })
}

fn share(v: &[usize]) -> Share {
    let at = |i: usize| v.get(i).copied().unwrap_or(0);
    Share {
        messages: at(0),
        messages_equal: at(1),
        rows: at(2),
        rows_equal: at(3),
        ..Share::default()
    }
}

impl Record {
    /// Reads a result file that `Run::to_toml` wrote.
    pub(crate) fn parse(text_: &str) -> Result<Record, String> {
        let t = toml::parse(text_).map_err(|e| e.to_string())?;
        let unstable = match t.get("unstable") {
            Some(Value::Array(a)) => {
                a.iter().filter_map(Value::as_str).map(str::to_string).collect()
            }
            _ => Vec::new(),
        };
        let checked = t.contains_key("messages").then(|| {
            let total = |k: &str| t.get(k).and_then(Value::as_int).unwrap_or(0) as usize;
            let cases = t
                .get("cases")
                .and_then(Value::as_table)
                .map(|c| c.iter().map(|(k, v)| (k.clone(), share(&ints(Some(v))))).collect())
                .unwrap_or_default();
            Checked {
                items: counts(t.get("exclusions").and_then(Value::as_table)),
                rules: counts(t.get("rules").and_then(Value::as_table)),
                share: Share {
                    unstable_messages: total("unstable_messages"),
                    ..share(&[
                        total("messages"),
                        total("messages_equal"),
                        total("rows"),
                        total("rows_equal"),
                    ])
                },
                cases,
            }
        });
        Ok(Record {
            suite: text(&t, "suite")?,
            version: int(&t, "version")? as u32,
            commit: text(&t, "commit")?,
            server: text(&t, "server")?,
            side: text(&t, "side")?,
            started: text(&t, "started")?,
            seconds: int(&t, "seconds")? as u64,
            total: int(&t, "total")? as usize,
            passed: int(&t, "passed")? as usize,
            unstable,
            checked,
        })
    }
}

/// Reads every result file of the work directory, in the order of version and file name.
pub(crate) fn load(work: &Path) -> Result<Vec<Record>, String> {
    let dir = work.join("results");
    let mut files = Vec::new();
    let Ok(versions) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    for v in versions.flatten() {
        for f in fs::read_dir(v.path()).map_err(|e| format!("{}: {e}", v.path().display()))? {
            let path = f.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "toml") {
                files.push(path);
            }
        }
    }
    let mut out = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        out.push(Record::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    out.sort_by(|a, b| (b.version, &a.suite, &a.side).cmp(&(a.version, &b.suite, &b.side)));
    Ok(out)
}

/// A number of the report: the count that is equal to the oracle, of the count of the oracle.
pub(crate) type Number = (usize, usize);

/// One row of a level table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Row {
    /// L1 to L5, as 0 to 4.
    pub(crate) level: usize,
    /// The key of the row in `ratchet.toml`, without the version.
    pub(crate) key: String,
    /// What the number counts.
    pub(crate) unit: &'static str,
    pub(crate) server: Option<Number>,
    pub(crate) oracle: Option<Number>,
    /// The number of the server under test that the ratchet checks: the count of the oracle less the differences. An unstable case is not a difference, so a run with more unstable cases does not lower it.
    pub(crate) count: Option<usize>,
    /// The case of the row was unstable in the run of the server under test, so the row has no number and the ratchet does not check it.
    pub(crate) unstable: bool,
    /// The case of the row was unstable in the run of the oracle against itself.
    pub(crate) oracle_unstable: bool,
}

/// The rows that spec/21 section 21.15 asks for and that M0 does not run yet. They show as "not run".
const NOT_RUN: [(usize, &str, &str); 4] = [
    (0, "client traces", "messages"),
    (2, "differential cases", "cases"),
    (3, "Hermitage cases", "cases"),
    (4, "client suites", "tests"),
];

/// The rows of one version.
pub(crate) fn rows(records: &[Record], version: u32) -> Vec<Row> {
    let mut rows: BTreeMap<(usize, String), Row> = BTreeMap::new();
    // `n` is the published number and `count` is the number of the ratchet.
    let mut put = |level: usize,
                   key: String,
                   unit: &'static str,
                   r: &Record,
                   n: Option<Number>,
                   count: usize| {
        let row =
            rows.entry((level, key.clone())).or_insert(Row { level, key, unit, ..Row::default() });
        if r.side == "server" {
            row.server = n;
            row.count = n.map(|_| count);
            row.unstable = n.is_none();
        } else {
            row.oracle = n;
            row.oracle_unstable = n.is_none();
        }
    };
    for r in records.iter().filter(|r| r.version == version) {
        let tests = Some((r.passed, r.total));
        let count = r.passed + r.unstable.len();
        match r.suite.as_str() {
            "pipeline" => put(0, "libpq_pipeline".into(), "tests", r, tests, count),
            "catalog" => {
                put(1, "catalog relations".into(), "relations", r, tests, count);
                for (name, s) in r.checked.iter().flat_map(|c| &c.cases) {
                    let n = Some((s.rows_equal, s.rows));
                    put(1, format!("rows of {name}"), "rows", r, n, s.rows_equal);
                }
                for name in &r.unstable {
                    put(1, format!("rows of {name}"), "rows", r, None, 0);
                }
            }
            "regress" => put(2, "regression tests".into(), "tests", r, tests, count),
            "isolation" => put(3, "isolation specs".into(), "specs", r, tests, count),
            "parameters" => put(3, "parameter cases".into(), "cases", r, tests, count),
            s => {
                if let (Some(name), Some(c)) = (s.strip_prefix("replay-"), &r.checked) {
                    let n = Some((c.share.messages_equal, c.share.messages));
                    let count = c.share.messages_equal + c.share.unstable_messages;
                    put(0, format!("trace {name}"), "messages", r, n, count);
                }
            }
        }
    }
    for (level, key, unit) in NOT_RUN {
        // The row of the client traces gives way to the rows of the traces that replay ran.
        if level == 0 && rows.keys().any(|(_, k)| k.starts_with("trace ")) {
            continue;
        }
        rows.entry((level, key.into())).or_insert(Row {
            level,
            key: key.into(),
            unit,
            ..Row::default()
        });
    }
    rows.into_values().collect()
}

fn number(n: Option<Number>) -> String {
    match n {
        Some((a, b)) if b > 0 => format!("{a} of {b} ({:.1}%)", 100.0 * a as f64 / b as f64),
        Some((a, b)) => format!("{a} of {b}"),
        None => "not run".into(),
    }
}

/// The ratchet: the best number of each row, by level and by `<version> <key>`.
pub(crate) type Ratchet = BTreeMap<String, BTreeMap<String, i64>>;

pub(crate) fn read_ratchet(text: &str) -> Result<Ratchet, String> {
    let t = toml::parse(text).map_err(|e| format!("ratchet.toml: {e}"))?;
    let mut out = Ratchet::new();
    for (level, v) in t {
        let table = v.as_table().ok_or(format!("ratchet.toml: {level} is not a table"))?;
        let mut rows = BTreeMap::new();
        for (key, n) in table {
            rows.insert(
                key.clone(),
                n.as_int().ok_or(format!("ratchet.toml: {level}.{key} is not an integer"))?,
            );
        }
        out.insert(level, rows);
    }
    Ok(out)
}

/// Writes the ratchet with the comment lines at the top of the old file.
pub(crate) fn write_ratchet(old: &str, r: &Ratchet) -> String {
    let mut text: String =
        old.lines().take_while(|l| l.starts_with('#')).map(|l| format!("{l}\n")).collect();
    for (level, rows) in r {
        text.push_str(&format!("\n[{level}]\n"));
        for (key, n) in rows {
            text.push_str(&format!("{} = {n}\n", quote(key)));
        }
    }
    text
}

const LEVEL_IDS: [&str; 5] = ["L1", "L2", "L3", "L4", "L5"];

/// The result of the ratchet check.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Check {
    /// The rows below their ratchet or without a number: level, key, ratchet, number.
    pub(crate) below: Vec<(String, String, i64, Option<usize>)>,
    /// The rows above their ratchet or new: level, key, ratchet, number.
    pub(crate) above: Vec<(String, String, Option<i64>, usize)>,
}

/// Checks the numbers of the server under test against the ratchet. Only the versions in `versions` are checked.
pub(crate) fn check(ratchet: &Ratchet, rows: &[(u32, Row)], versions: &[u32]) -> Check {
    let mut now: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut unstable: Vec<(String, String)> = Vec::new();
    for (v, row) in rows {
        let key = (LEVEL_IDS[row.level].to_string(), format!("{v} {}", row.key));
        if let Some(n) = row.count {
            now.insert(key, n);
        } else if row.unstable {
            unstable.push(key);
        }
    }
    let mut c = Check::default();
    for (level, entries) in ratchet {
        for (key, &best) in entries {
            let in_scope = key
                .split_once(' ')
                .and_then(|(v, _)| v.parse::<u32>().ok())
                .is_some_and(|v| versions.contains(&v));
            if !in_scope {
                continue;
            }
            match now.get(&(level.clone(), key.clone())) {
                Some(&n) if n as i64 >= best => {
                    if n as i64 > best {
                        c.above.push((level.clone(), key.clone(), Some(best), n));
                    }
                }
                None if unstable.contains(&(level.clone(), key.clone())) => {}
                n => c.below.push((level.clone(), key.clone(), best, n.copied())),
            }
        }
    }
    for ((level, key), n) in now {
        if !ratchet.get(&level).is_some_and(|r| r.contains_key(&key)) {
            c.above.push((level, key, None, n));
        }
    }
    c
}

/// Raises the ratchet to each number of `check.above`.
pub(crate) fn raise(ratchet: &mut Ratchet, check: &Check) {
    for (level, key, _, n) in &check.above {
        ratchet.entry(level.clone()).or_default().insert(key.clone(), *n as i64);
    }
}

/// The text of `report.md`.
pub(crate) fn markdown(
    date: &str,
    pins: &Pins,
    records: &[Record],
    versions: &[u32],
    work: &Path,
    check: &Check,
) -> String {
    let mut m = format!("# rupg-compat report {date}\n\n");
    m.push_str(&format!(
        "This report reads {} result files in `{}`. Each number is the count that is equal to the oracle, of the count that the oracle gives. The server column is the server under test, and the oracle column is the oracle against itself. The unstable cases of the oracle are not counted (spec/21 section 21.1). The ratchet checks the count of the oracle less the differences, with each unstable case as no difference, so that a run with more unstable cases does not fail it. A relation that was unstable in the run of the server under test shows as unstable, and the ratchet does not check its rows in that run.\n\n",
        records.len(),
        work.join("results").display()
    ));
    m.push_str("## Pins\n\n| Pin | Release | Commit |\n|---|---|---|\n");
    for p in &pins.postgres {
        m.push_str(&format!("| PostgreSQL {} | {} | `{}` |\n", p.major, p.release, p.commit));
    }
    m.push_str(&format!("| rupg | | `{}` |\n", pins.rupg_commit));
    if pins.clients.is_empty() {
        m.push_str("\n`pins.toml` has no client yet.\n");
    } else {
        m.push_str("\n| Client | Version |\n|---|---|\n");
        for c in &pins.clients {
            m.push_str(&format!("| {} | {} |\n", c.name, c.version));
        }
    }
    let levels = crate::levels::LEVELS;
    for &v in versions {
        let recs: Vec<&Record> = records.iter().filter(|r| r.version == v).collect();
        let kind =
            if v == pins.reference { "the reference" } else { "a shim of `rupg.compat_version`" };
        m.push_str(&format!("\n## PostgreSQL {v}\n\nPostgreSQL {v} is {kind}.\n\n"));
        if recs.is_empty() {
            m.push_str("No run has a result file for this version.\n");
            continue;
        }
        m.push_str("| Run | Side | Server | PostgreSQL commit | Started | Seconds | Unstable |\n|---|---|---|---|---|---|---|\n");
        for r in &recs {
            m.push_str(&format!(
                "| {} | {} | {} | `{}` | {} | {} | {} |\n",
                r.suite,
                r.side,
                r.server,
                r.commit,
                r.started,
                r.seconds,
                r.unstable.len()
            ));
        }
        let rows = rows(records, v);
        for (i, level) in levels.iter().enumerate() {
            m.push_str(&format!(
                "\n### {} {}\n\n| Row | Unit | Server | Oracle |\n|---|---|---|---|\n",
                level.id,
                level.name.split(':').next().unwrap_or_default()
            ));
            let main = rows.iter().filter(|r| r.level == i && !r.key.starts_with("rows of "));
            for r in main {
                m.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    r.key,
                    r.unit,
                    number(r.server),
                    number(r.oracle)
                ));
            }
            let each: Vec<&Row> =
                rows.iter().filter(|r| r.level == i && r.key.starts_with("rows of ")).collect();
            if !each.is_empty() {
                m.push_str(
                    "\nThe rows of each relation:\n\n| Relation | Server | Oracle |\n|---|---|---|\n",
                );
                for r in each {
                    let show = |unstable: bool, n: Option<Number>| {
                        if unstable { "unstable".into() } else { number(n) }
                    };
                    m.push_str(&format!(
                        "| {} | {} | {} |\n",
                        r.key.trim_start_matches("rows of "),
                        show(r.unstable, r.server),
                        show(r.oracle_unstable, r.oracle)
                    ));
                }
            }
        }
        m.push_str("\n### Exclusions\n\nEach exclusion of spec/05 section 5.3 and each rule of section 5.2 that a run used, with the count of values that it applied to and the count of those that differed.\n\n");
        let mut any = false;
        let mut table = String::from(
            "| Run | Side | Exclusion or rule | Applied | Differed |\n|---|---|---|---|---|\n",
        );
        for r in &recs {
            let Some(c) = &r.checked else { continue };
            let items = compare::ITEMS
                .iter()
                .enumerate()
                .map(|(i, n)| (format!("{}. {n}", i + 1), c.items[i]));
            let rules = compare::RULES.iter().zip(c.rules).map(|(n, k)| (format!("rule: {n}"), k));
            for (name, k) in items.chain(rules).filter(|(_, k)| k.applied > 0) {
                any = true;
                table.push_str(&format!(
                    "| {} | {} | {name} | {} | {} |\n",
                    r.suite, r.side, k.applied, k.differed
                ));
            }
        }
        m.push_str(if any { &table } else { "No run of this version used an exclusion.\n" });
    }
    m.push_str("\n## Ratchet\n");
    if check.below.is_empty() && check.above.is_empty() {
        m.push_str("\nEach number of the server under test is equal to its ratchet.\n");
    }
    if !check.below.is_empty() {
        m.push_str("\nThese rows are below `ratchet.toml`. A number may go down only with an entry in spec/24 of tamnd/rupg that says why.\n\n| Level | Row | Ratchet | Now |\n|---|---|---|---|\n");
        for (level, key, best, n) in &check.below {
            let now = n.map_or("no result".into(), |n| n.to_string());
            m.push_str(&format!("| {level} | {key} | {best} | {now} |\n"));
        }
    }
    if !check.above.is_empty() {
        m.push_str("\nThese rows are above `ratchet.toml` or are not in it. `rupg-compat report --raise` writes them to the ratchet.\n\n| Level | Row | Ratchet | Now |\n|---|---|---|---|\n");
        for (level, key, best, n) in &check.above {
            let best = best.map_or("none".into(), |b| b.to_string());
            m.push_str(&format!("| {level} | {key} | {best} | {n} |\n"));
        }
    }
    m
}

/// The directory of a report: `reports/<date>/` under `base`.
pub(crate) fn report_dir(base: &Path, date: &str) -> PathBuf {
    base.join(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(suite: &str, side: &str, passed: usize, total: usize) -> Record {
        Record {
            suite: suite.into(),
            version: 19,
            side: side.into(),
            passed,
            total,
            ..Record::default()
        }
    }

    #[test]
    fn a_result_file_reads_back() {
        let run = crate::suites::Run {
            suite: "catalog".into(),
            tests: vec![("pg_class".into(), true), ("pg_am\"x".into(), false)],
            unstable: vec!["pg_locks".into()],
            seconds: 7,
            checked: Some(Checked {
                items: std::array::from_fn(|i| Count { applied: i, differed: 0 }),
                rules: [Count { applied: 2, differed: 1 }; 4],
                share: Share {
                    messages: 9,
                    messages_equal: 8,
                    rows: 5,
                    rows_equal: 4,
                    unstable_messages: 3,
                },
                cases: vec![
                    (
                        "pg_class".into(),
                        Share {
                            messages: 4,
                            messages_equal: 4,
                            rows: 2,
                            rows_equal: 2,
                            unstable_messages: 0,
                        },
                    ),
                    (
                        "pg_am\"x".into(),
                        Share {
                            messages: 5,
                            messages_equal: 4,
                            rows: 3,
                            rows_equal: 2,
                            unstable_messages: 0,
                        },
                    ),
                ],
            }),
        };
        let pins = Pins::builtin();
        let o = crate::oracle::Oracle::new(&pins, 19, Path::new("/w")).unwrap();
        let server =
            crate::suites::Server { addr: "127.0.0.1:5432".parse().unwrap(), oracle: false };
        let r = Record::parse(&run.to_toml(&o, &server, "2026-10-07T00:00:00Z")).unwrap();
        assert_eq!((r.suite.as_str(), r.version, r.side.as_str()), ("catalog", 19, "server"));
        assert_eq!((r.passed, r.total, r.seconds), (1, 2, 7));
        assert_eq!(r.unstable, ["pg_locks"]);
        let mut want = run.checked.unwrap();
        want.cases.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(r.checked.unwrap(), want);
    }

    #[test]
    fn the_rows_of_a_version() {
        let mut catalog = record("catalog", "server", 1, 2);
        catalog.unstable = vec!["pg_locks".into()];
        catalog.checked = Some(Checked {
            cases: vec![("pg_class".into(), Share { rows: 3, rows_equal: 2, ..Share::default() })],
            ..Checked::default()
        });
        let mut replay = record("replay-psql", "server", 1, 1);
        replay.checked = Some(Checked {
            share: Share {
                messages: 10,
                messages_equal: 9,
                unstable_messages: 4,
                ..Share::default()
            },
            ..Checked::default()
        });
        let recs = [
            record("regress", "oracle", 239, 239),
            record("regress", "server", 200, 239),
            catalog,
            replay,
            record("isolation", "server", 1, 135),
        ];
        let got = rows(&recs, 19);
        let find = |key: &str| got.iter().find(|r| r.key == key).unwrap();
        assert_eq!(find("regression tests").server, Some((200, 239)));
        assert_eq!(find("regression tests").oracle, Some((239, 239)));
        assert_eq!(find("regression tests").count, Some(200));
        assert_eq!(find("rows of pg_class").server, Some((2, 3)));
        assert_eq!(find("rows of pg_class").count, Some(2));
        assert!(find("rows of pg_locks").unstable);
        assert!(!find("rows of pg_locks").oracle_unstable);
        assert_eq!(find("rows of pg_locks").count, None);
        assert_eq!(find("catalog relations").level, 1);
        // The unstable relation is not a difference for the ratchet.
        assert_eq!(find("catalog relations").count, Some(2));
        assert_eq!(find("trace psql").server, Some((9, 10)));
        assert_eq!(find("trace psql").count, Some(13));
        assert_eq!(find("isolation specs").level, 3);
        assert_eq!(find("Hermitage cases").server, None);
        assert!(got.iter().all(|r| r.key != "client traces"));
        assert!(rows(&recs, 18).iter().all(|r| r.server.is_none() && r.oracle.is_none()));
    }

    #[test]
    fn the_ratchet_fails_a_lower_number() {
        let ratchet = read_ratchet(
            "# best\n[L3]\n\"19 regression tests\" = 210\n[L4]\n\"19 isolation specs\" = 100\n\"18 isolation specs\" = 5\n",
        )
        .unwrap();
        let row = |level: usize, key: &str, n: usize| Row {
            level,
            key: key.into(),
            server: Some((n, 300)),
            count: Some(n),
            ..Row::default()
        };
        let rows = [(19, row(2, "regression tests", 200)), (19, row(3, "parameter cases", 7))];
        let c = check(&ratchet, &rows, &[19]);
        assert_eq!(
            c.below,
            [
                ("L3".into(), "19 regression tests".into(), 210, Some(200)),
                ("L4".into(), "19 isolation specs".into(), 100, None)
            ]
        );
        assert_eq!(c.above, [("L4".into(), "19 parameter cases".into(), None, 7)]);
        let unstable =
            Row { level: 3, key: "isolation specs".into(), unstable: true, ..Row::default() };
        // An unstable row is not checked, so only the regression row has no result.
        assert_eq!(check(&ratchet, &[(19, unstable)], &[19]).below.len(), 1);
        let rows = [(19, row(2, "regression tests", 220)), (19, row(3, "isolation specs", 100))];
        let c = check(&ratchet, &rows, &[19]);
        assert!(c.below.is_empty());
        let mut raised = ratchet.clone();
        raise(&mut raised, &c);
        let text = write_ratchet("# best\n[L3]\n", &raised);
        assert_eq!(read_ratchet(&text).unwrap()["L3"]["19 regression tests"], 220);
        assert!(text.starts_with("# best\n\n[L3]\n"));
    }
}
