//! The report and the ratchet (spec/21 section 21.15).
//!
//! `rupg-compat report` reads the result files `results/N/*.toml` of the work directory and writes `reports/<date>/report.md`, with one table for each level and each version. Then it checks each row of `ratchet.toml`: the report fails when a number of the server under test is below its ratchet.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compare::{self, Count, Share};
use crate::pins::Pins;
use crate::resources::Resources;
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
    /// The tests that fail.
    pub(crate) fail: Vec<String>,
    /// The statement count of the regression suite: equal and total.
    pub(crate) statements: Option<Number>,
    pub(crate) checked: Option<Checked>,
    /// The peak memory and the CPU time of the server, for a run with `--cgroup`.
    pub(crate) resources: Option<Resources>,
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
        let names = |key: &str| match t.get(key) {
            Some(Value::Array(a)) => {
                a.iter().filter_map(Value::as_str).map(str::to_string).collect()
            }
            _ => Vec::new(),
        };
        let resources = match (t.get("peak_memory"), t.get("cpu_usec")) {
            (Some(Value::Int(peak)), Some(Value::Int(cpu))) => Some(Resources {
                peak: *peak as u64,
                cpu_usec: *cpu as u64,
                reset: matches!(t.get("peak_reset"), Some(Value::Bool(true))),
            }),
            _ => None,
        };
        let statements = match (t.get("statements_equal"), t.get("statements")) {
            (Some(Value::Int(equal)), Some(Value::Int(total))) => {
                Some((*equal as usize, *total as usize))
            }
            _ => None,
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
            unstable: names("unstable"),
            fail: names("fail"),
            statements,
            checked,
            resources,
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

/// The tools of spec/21 section 21.4.2 that CI cannot run, each with the driver that it uses. L1 shows the share of the trace of the driver for each tool.
const DRIVER_TRACES: [(&str, &str); 6] = [
    ("datagrip", "pgjdbc"),
    ("looker", "pgjdbc"),
    ("power-bi", "npgsql"),
    ("postico", "libpq"),
    ("tableau", "psqlodbc"),
    ("tableplus", "libpq"),
];

/// The catalogs of L2 tier 1 (spec/05 section 5.1). `version()`, `format_type` and the `reg*` casts of tier 1 are functions, and the gate and type suites check them.
const TIER_1: [&str; 4] = ["pg_type", "pg_namespace", "pg_class", "pg_attribute"];

/// The catalogs of L2 tier 2 (spec/05 section 5.1). Each relation of `information_schema` is in tier 2 too.
const TIER_2: [&str; 7] =
    ["pg_index", "pg_constraint", "pg_attrdef", "pg_description", "pg_enum", "pg_range", "pg_proc"];

/// The tier of a relation of the catalog suite, or `None` when the relation is in no tier.
fn tier(name: &str) -> Option<usize> {
    let (schema, table) = name.split_once('.').unwrap_or(("pg_catalog", name));
    if schema == "information_schema" {
        return Some(2);
    }
    if schema != "pg_catalog" {
        return None;
    }
    if TIER_1.contains(&table) {
        Some(1)
    } else if TIER_2.contains(&table) {
        Some(2)
    } else {
        None
    }
}

/// The rows of each tier of a catalog run, with the key suffix of the suite. The number is the sum of the equal rows of the relations of the tier, of the sum of their rows. An unstable relation of a tier makes the tier unstable.
fn tier_rows(r: &Record, suffix: &str) -> Vec<(String, Option<Number>)> {
    let mut out = Vec::new();
    for t in [1, 2] {
        let key = format!("L2 tier {t} rows{suffix}");
        if r.unstable.iter().any(|n| tier(n) == Some(t)) {
            out.push((key, None));
            continue;
        }
        let mut n = (0, 0);
        for (_, s) in r.checked.iter().flat_map(|c| &c.cases).filter(|(n, _)| tier(n) == Some(t)) {
            n.0 += s.rows_equal;
            n.1 += s.rows;
        }
        out.push((key, Some(n)));
    }
    out
}

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
            "version" => put(0, "version forms".into(), "checks", r, tests, count),
            "catalog" => {
                put(1, "catalog relations".into(), "relations", r, tests, count);
                for (key, n) in tier_rows(r, "") {
                    put(1, key, "rows", r, n, n.map_or(0, |n| n.0));
                }
                for (name, s) in r.checked.iter().flat_map(|c| &c.cases) {
                    let n = Some((s.rows_equal, s.rows));
                    put(1, format!("rows of {name}"), "rows", r, n, s.rows_equal);
                }
                for name in &r.unstable {
                    put(1, format!("rows of {name}"), "rows", r, None, 0);
                }
            }
            "catalog-gate" => {
                put(
                    1,
                    "catalog relations with the gate schema".into(),
                    "relations",
                    r,
                    tests,
                    count,
                );
                for (key, n) in tier_rows(r, " with the gate schema") {
                    put(1, key, "rows", r, n, n.map_or(0, |n| n.0));
                }
                for (name, s) in r.checked.iter().flat_map(|c| &c.cases) {
                    let n = Some((s.rows_equal, s.rows));
                    put(
                        1,
                        format!("rows of {name} with the gate schema"),
                        "rows",
                        r,
                        n,
                        s.rows_equal,
                    );
                }
                for name in &r.unstable {
                    put(1, format!("rows of {name} with the gate schema"), "rows", r, None, 0);
                }
            }
            "catalog-corpus" => {
                put(1, "catalog queries of the client traces".into(), "queries", r, tests, count);
            }
            "dump" => put(1, "pg_dump round trip".into(), "checks", r, tests, count),
            "gate" => put(1, "gate G1".into(), "clients", r, tests, count),
            "connect" => put(0, "connect with SCRAM over TLS".into(), "clients", r, tests, count),
            "regress" => {
                put(2, "regression tests".into(), "tests", r, tests, count);
                if r.statements.is_some() {
                    put(2, "regression statements".into(), "statements", r, r.statements, 0);
                }
            }
            "types" => put(2, "type input and output cases".into(), "cases", r, tests, count),
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
    // The statement count shows progress inside a test that fails. It is never used for a gate (spec/05 section 5.8), so the ratchet does not check it.
    if let Some(row) = rows.get_mut(&(2, "regression statements".into())) {
        row.count = None;
    }
    // A tool row copies the numbers of its driver. It has no count of its own, so the ratchet checks the driver only.
    for (tool, driver) in DRIVER_TRACES {
        if let Some(d) = rows.get(&(0, format!("trace {driver}"))).cloned() {
            let key = format!("trace {tool} through {driver}");
            rows.insert((0, key.clone()), Row { key, count: None, ..d });
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
        "This report reads {} result files in `{}`. Each number is the count that is equal to the oracle, of the count that the oracle gives. The server column is the server under test, and the oracle column is the oracle against itself. The unstable cases of the oracle are not counted (spec/21 section 21.1). The regression statements row is the statement count of spec/05 section 5.8: the statements whose output is equal to the output of the oracle. It shows progress inside a test that fails, and the ratchet does not check it. The ratchet checks the count of the oracle less the differences, with each unstable case as no difference, so that a run with more unstable cases does not fail it. A relation that was unstable in the run of the server under test shows as unstable, and the ratchet does not check its rows in that run.\n\n",
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
        m.push_str(&resources(&recs));
        m.push_str(&failing(&recs));
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

/// The resource table of one version: for each suite, the peak memory and the CPU time of the server under test and of the oracle.
fn resources(recs: &[&Record]) -> String {
    let mut m = String::from(
        "\n### Resources\n\nThe peak memory and the CPU time of the server of each run, from its cgroup v2 (spec/02 section 2.6.1 of tamnd/rupg). The ratio is the number of the oracle over the number of rupg, so a ratio above 1 is better for rupg. A peak with * is the peak since the cgroup started, because the kernel cannot reset it before Linux 6.12.\n\n",
    );
    let side = |suite: &str, side: &str| {
        recs.iter().find(|r| r.suite == suite && r.side == side).and_then(|r| r.resources)
    };
    let mib = |r: Option<Resources>| {
        r.map_or("not measured".into(), |r| {
            format!("{:.1} MiB{}", r.peak as f64 / 1048576.0, if r.reset { "" } else { " *" })
        })
    };
    let cpu = |r: Option<Resources>| {
        r.map_or("not measured".into(), |r| format!("{:.2} s", r.cpu_usec as f64 / 1e6))
    };
    let ratio = |a: Option<u64>, b: Option<u64>| match (a, b) {
        (Some(o), Some(s)) if s > 0 => format!("{:.2}", o as f64 / s as f64),
        _ => "-".into(),
    };
    let mut suites: Vec<&str> = recs.iter().map(|r| r.suite.as_str()).collect();
    suites.dedup();
    let mut table = String::from(
        "| Run | rupg peak | Oracle peak | Ratio | rupg CPU | Oracle CPU | Ratio |\n|---|---|---|---|---|---|---|\n",
    );
    let mut any = false;
    for suite in suites {
        let (s, o) = (side(suite, "server"), side(suite, "oracle"));
        if s.is_none() && o.is_none() {
            continue;
        }
        any = true;
        table.push_str(&format!(
            "| {suite} | {} | {} | {} | {} | {} | {} |\n",
            mib(s),
            mib(o),
            ratio(o.map(|r| r.peak), s.map(|r| r.peak)),
            cpu(s),
            cpu(o),
            ratio(o.map(|r| r.cpu_usec), s.map(|r| r.cpu_usec)),
        ));
    }
    m.push_str(if any {
        &table
    } else {
        "No run of this version has resource numbers. A run with `--cgroup` has them.\n"
    });
    m
}

/// The tests of the server under test that fail, for each run of one version.
fn failing(recs: &[&Record]) -> String {
    let mut m = String::from("\n### Failing items\n\n");
    let mut any = false;
    for r in recs.iter().filter(|r| r.side == "server" && !r.fail.is_empty()) {
        any = true;
        let names: Vec<String> = r.fail.iter().map(|n| format!("`{n}`")).collect();
        m.push_str(&format!(
            "{} ({} of {}): {}\n\n",
            r.suite,
            r.fail.len(),
            r.total,
            names.join(", ")
        ));
    }
    if !any {
        m.push_str("No test of the server under test fails.\n");
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
    fn the_statement_count_has_a_row_that_the_ratchet_does_not_check() {
        let text = "suite = \"regress\"\nversion = 19\ncommit = \"c\"\nserver = \"s\"\nside = \"server\"\nstarted = \"t\"\nseconds = 1\ntotal = 2\npassed = 1\npass = [\"int4\"]\nfail = [\"join\"]\nunstable = []\nstatements = 40\nstatements_equal = 31\n";
        let server = Record::parse(text).unwrap();
        assert_eq!(server.statements, Some((31, 40)));
        let oracle = Record { side: "oracle".into(), statements: Some((40, 40)), ..server.clone() };
        let rows = rows(&[oracle, server], 19);
        let row = rows.iter().find(|r| r.key == "regression statements").unwrap();
        assert_eq!(
            (row.level, row.server, row.oracle, row.count),
            (2, Some((31, 40)), Some((40, 40)), None)
        );
        let check =
            check(&Ratchet::new(), &rows.into_iter().map(|r| (19, r)).collect::<Vec<_>>(), &[19]);
        assert!(check.above.iter().all(|(_, key, _, _)| key != "19 regression statements"));
    }

    #[test]
    fn the_report_has_resources_and_failing_items() {
        let mut server = Record::parse(
            "suite = \"regress\"\nversion = 19\ncommit = \"c\"\nserver = \"s\"\nside = \"server\"\nstarted = \"t\"\nseconds = 1\ntotal = 3\npassed = 1\npass = [\"int4\"]\nfail = [\"join\", \"rules\"]\nunstable = []\npeak_memory = 1048576\ncpu_usec = 500000\npeak_reset = false\n",
        )
        .unwrap();
        assert_eq!(server.fail, ["join", "rules"]);
        assert_eq!(
            server.resources,
            Some(Resources { peak: 1048576, cpu_usec: 500000, reset: false })
        );
        let mut oracle = server.clone();
        oracle.side = "oracle".into();
        oracle.fail.clear();
        oracle.resources = Some(Resources { peak: 4194304, cpu_usec: 1000000, reset: true });
        let text = resources(&[&oracle, &server]);
        assert!(
            text.contains("| regress | 1.0 MiB * | 4.0 MiB | 4.00 | 0.50 s | 1.00 s | 2.00 |"),
            "{text}"
        );
        assert!(failing(&[&oracle, &server]).contains("regress (2 of 3): `join`, `rules`"));
        server.resources = None;
        server.fail.clear();
        oracle.resources = None;
        assert!(
            resources(&[&oracle, &server]).contains("No run of this version has resource numbers")
        );
        assert!(failing(&[&server]).contains("No test of the server under test fails."));
    }

    #[test]
    fn a_result_file_reads_back() {
        let run = crate::suites::Run {
            suite: "catalog".into(),
            tests: vec![("pg_class".into(), true), ("pg_am\"x".into(), false)],
            unstable: vec!["pg_locks".into()],
            seconds: 7,
            statements: None,
            checked: Some(Checked {
                items: std::array::from_fn(|i| Count { applied: i, differed: 0 }),
                rules: [Count { applied: 2, differed: 1 }; 5],
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
    fn a_tool_row_copies_its_driver() {
        let mut replay = record("replay-libpq", "server", 1, 1);
        replay.checked = Some(Checked {
            share: Share { messages: 10, messages_equal: 9, ..Share::default() },
            ..Checked::default()
        });
        let got = rows(&[replay], 19);
        let find = |key: &str| got.iter().find(|r| r.key == key);
        let tool = find("trace tableplus through libpq").unwrap();
        assert_eq!(tool.server, Some((9, 10)));
        assert_eq!(tool.count, None);
        assert!(find("trace postico through libpq").is_some());
        // pgjdbc did not run, so its tools have no row.
        assert!(find("trace datagrip through pgjdbc").is_none());
    }

    #[test]
    fn the_rows_of_a_version() {
        let mut catalog = record("catalog", "server", 1, 2);
        catalog.unstable = vec!["pg_locks".into()];
        catalog.checked = Some(Checked {
            cases: vec![
                ("pg_class".into(), Share { rows: 3, rows_equal: 2, ..Share::default() }),
                ("pg_catalog.pg_type".into(), Share { rows: 4, rows_equal: 4, ..Share::default() }),
                ("pg_catalog.pg_proc".into(), Share { rows: 6, rows_equal: 5, ..Share::default() }),
                (
                    "information_schema.tables".into(),
                    Share { rows: 2, rows_equal: 2, ..Share::default() },
                ),
                ("pg_catalog.pg_am".into(), Share { rows: 9, rows_equal: 1, ..Share::default() }),
            ],
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
        let mut gate_catalog = record("catalog-gate", "server", 3, 4);
        gate_catalog.checked = Some(Checked {
            cases: vec![("pg_class".into(), Share { rows: 5, rows_equal: 5, ..Share::default() })],
            ..Checked::default()
        });
        let recs = [
            record("regress", "oracle", 239, 239),
            record("regress", "server", 200, 239),
            catalog,
            gate_catalog,
            replay,
            record("isolation", "server", 1, 135),
            record("dump", "server", 6, 7),
            record("gate", "server", 5, 8),
            record("version", "server", 8, 8),
        ];
        let got = rows(&recs, 19);
        let find = |key: &str| got.iter().find(|r| r.key == key).unwrap();
        assert_eq!(find("regression tests").server, Some((200, 239)));
        assert_eq!(find("regression tests").oracle, Some((239, 239)));
        assert_eq!(find("regression tests").count, Some(200));
        assert!(got.iter().all(|r| r.key != "regression statements"));
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
        assert_eq!(find("pg_dump round trip").level, 1);
        assert_eq!(find("pg_dump round trip").server, Some((6, 7)));
        assert_eq!(find("gate G1").level, 1);
        assert_eq!(find("gate G1").server, Some((5, 8)));
        assert_eq!(find("version forms").level, 0);
        assert_eq!(find("version forms").server, Some((8, 8)));
        assert_eq!(find("catalog relations with the gate schema").server, Some((3, 4)));
        assert_eq!(find("rows of pg_class with the gate schema").server, Some((5, 5)));
        assert_eq!(find("L2 tier 1 rows").server, Some((6, 7)));
        assert_eq!(find("L2 tier 1 rows").count, Some(6));
        assert_eq!(find("L2 tier 2 rows").server, Some((7, 8)));
        assert_eq!(find("L2 tier 1 rows with the gate schema").server, Some((5, 5)));
        assert_eq!(find("L2 tier 2 rows with the gate schema").server, Some((0, 0)));
        assert_eq!(find("Hermitage cases").server, None);
        assert!(got.iter().all(|r| r.key != "client traces"));
        assert!(rows(&recs, 18).iter().all(|r| r.server.is_none() && r.oracle.is_none()));
    }

    #[test]
    fn an_unstable_relation_makes_its_tier_unstable() {
        let mut catalog = record("catalog", "server", 1, 2);
        catalog.unstable = vec!["pg_catalog.pg_class".into()];
        let got = rows(&[catalog], 19);
        let find = |key: &str| got.iter().find(|r| r.key == key).unwrap();
        assert!(find("L2 tier 1 rows").unstable);
        assert_eq!(find("L2 tier 1 rows").count, None);
        assert_eq!(find("L2 tier 2 rows").server, Some((0, 0)));
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
