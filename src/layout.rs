//! The layout equivalence oracle of spec/21 section 21.4.5. A rupg table can have rows in the hot store, in the cold store, or in both. The seed makes three tables and their rows, and the harness loads them in each layout of the spec in a new database. Then the same queries run on each layout and on the oracle, and each answer must be the answer of the oracle on the same data.
//!
//! The layouts use the controls of the spec. A cold row comes from a `COPY` into a table `USING columnar`, which writes segments directly (spec/04 section 4.4, spec/10 section 10.8). A hot row comes from `INSERT`. "No summaries" sets `rupg.use_summaries = off` (spec/12 section 12.4). When the server does not accept a step of the setup of a layout, the layout is not available, and the run gives the error.

use std::time::Duration;

use crate::client::Conn;
use crate::generate::{self, Rng, Sides, Table};
use crate::logic;
use crate::slt::{self, Sort};

/// The layouts, in the order of the report.
pub(crate) const LAYOUTS: [&str; 6] =
    ["hot only", "cold only", "mixed", "mixed after change", "no summaries", "many workers"];

/// The depth of a predicate.
const DEPTH: usize = 3;
/// The rows of one `INSERT`.
const CHUNK: usize = 100;

/// The tables, the rows and the changes of a seed.
pub(crate) struct Data {
    pub(crate) tables: Vec<Table>,
    /// The SQL literals of the rows of each table.
    rows: Vec<Vec<Vec<String>>>,
    /// For each row, true when the row goes to the cold store in a mixed layout.
    cold: Vec<Vec<bool>>,
    /// The updates, deletes and inserts after the load.
    pub(crate) changes: Vec<String>,
}

fn value(ty: &str, rng: &mut Rng) -> String {
    if rng.below(8) == 0 {
        return "NULL".into();
    }
    match ty {
        "int4" | "int8" => (rng.below(2001) as i64 - 1000).to_string(),
        "text" => {
            rng.pick(&["'a'", "''", "'b'", "'abc'", "'A'", "'ab'", "'b%'", "'x y'"]).to_string()
        }
        "bool" => rng.pick(&["true", "false"]).to_string(),
        _ => ((rng.below(4001) as f64 - 2000.0) / 4.0).to_string(),
    }
}

/// The field of a SQL literal in the text format of `COPY`. The literals of `value` have no tab and no backslash.
fn copy_field(literal: &str) -> &str {
    if literal == "NULL" {
        return "\\N";
    }
    literal.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')).unwrap_or(literal)
}

impl Data {
    /// Makes the tables of `generate` with an `id` column, `rows` rows in each table, and the changes.
    pub(crate) fn new(rng: &mut Rng, rows: usize) -> Data {
        let mut tables = generate::schema(rng).0;
        for t in &mut tables {
            t.cols.insert(0, ("id".into(), "int4"));
        }
        let mut data = Data { tables, rows: Vec::new(), cold: Vec::new(), changes: Vec::new() };
        for t in &data.tables {
            let list: Vec<Vec<String>> = (1..=rows)
                .map(|id| {
                    let mut row = vec![id.to_string()];
                    row.extend(t.cols[1..].iter().map(|(_, ty)| value(ty, rng)));
                    row
                })
                .collect();
            data.cold.push((0..rows).map(|_| rng.below(2) == 0).collect());
            data.rows.push(list);
        }
        for t in &data.tables {
            for _ in 0..rows.div_ceil(10) {
                let (col, ty) = &t.cols[1 + rng.below(t.cols.len() - 1)];
                let id = 1 + rng.below(rows);
                data.changes.push(format!(
                    "UPDATE {} SET {col} = {} WHERE id = {id}",
                    t.name,
                    value(ty, rng)
                ));
            }
            for _ in 0..rows.div_ceil(10) {
                data.changes.push(format!(
                    "DELETE FROM {} WHERE id = {}",
                    t.name,
                    1 + rng.below(rows)
                ));
            }
            for id in rows + 1..=rows + rows.div_ceil(20) {
                let mut row = vec![id.to_string()];
                row.extend(t.cols[1..].iter().map(|(_, ty)| value(ty, rng)));
                data.changes.push(format!("INSERT INTO {} VALUES ({})", t.name, row.join(", ")));
            }
        }
        data
    }

    fn create(&self, t: usize, columnar: bool) -> String {
        let defs: Vec<String> =
            self.tables[t].cols.iter().map(|(c, ty)| format!("{c} {ty}")).collect();
        let using = if columnar { " USING columnar" } else { "" };
        format!("CREATE TABLE {} ({}){using}", self.tables[t].name, defs.join(", "))
    }

    fn inserts(&self, t: usize, rows: &[&Vec<String>]) -> Vec<Step> {
        rows.chunks(CHUNK)
            .map(|chunk| {
                let values: Vec<String> =
                    chunk.iter().map(|r| format!("({})", r.join(", "))).collect();
                Step::Sql(format!(
                    "INSERT INTO {} VALUES {}",
                    self.tables[t].name,
                    values.join(", ")
                ))
            })
            .collect()
    }

    fn copy(&self, t: usize, rows: &[&Vec<String>]) -> Step {
        let mut text = String::new();
        for r in rows {
            let fields: Vec<&str> = r.iter().map(|v| copy_field(v)).collect();
            text.push_str(&fields.join("\t"));
            text.push('\n');
        }
        Step::Copy(format!("COPY {} FROM STDIN", self.tables[t].name), text)
    }

    /// The statements that load the data in a layout.
    pub(crate) fn setup(&self, layout: &str) -> Vec<Step> {
        let mut steps = Vec::new();
        if layout == "many workers" {
            for s in [
                "SET max_parallel_workers_per_gather = 4",
                "SET parallel_setup_cost = 0",
                "SET parallel_tuple_cost = 0",
                "SET min_parallel_table_scan_size = 0",
            ] {
                steps.push(Step::Sql(s.into()));
            }
        }
        for t in 0..self.tables.len() {
            let all: Vec<&Vec<String>> = self.rows[t].iter().collect();
            let (cold, hot): (Vec<_>, Vec<_>) =
                self.rows[t].iter().enumerate().partition(|(i, _)| self.cold[t][*i]);
            let cold: Vec<&Vec<String>> = cold.into_iter().map(|(_, r)| r).collect();
            let hot: Vec<&Vec<String>> = hot.into_iter().map(|(_, r)| r).collect();
            match layout {
                "hot only" | "many workers" => {
                    steps.push(Step::Sql(self.create(t, false)));
                    steps.extend(self.inserts(t, &all));
                }
                "cold only" => {
                    steps.push(Step::Sql(self.create(t, true)));
                    steps.push(self.copy(t, &all));
                }
                _ => {
                    steps.push(Step::Sql(self.create(t, true)));
                    steps.push(self.copy(t, &cold));
                    steps.extend(self.inserts(t, &hot));
                }
            }
        }
        if layout == "mixed after change" {
            steps.extend(self.changes.iter().map(|c| Step::Sql(c.clone())));
        }
        if layout == "no summaries" {
            steps.push(Step::Sql("SET rupg.use_summaries = off".into()));
        }
        steps
    }

    /// The SQL text of the data in the hot layout, for the file of the failures.
    pub(crate) fn sql(&self) -> Vec<String> {
        self.setup("hot only")
            .into_iter()
            .filter_map(|s| match s {
                Step::Sql(sql) => Some(sql),
                Step::Copy(..) => None,
            })
            .collect()
    }
}

/// A step of the setup of a layout.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    Sql(String),
    /// A `COPY FROM STDIN` and its data.
    Copy(String, String),
}

/// Runs the steps, and gives the first error.
pub(crate) fn load(c: &mut Conn, steps: &[Step]) -> Result<Option<String>, String> {
    for step in steps {
        let msgs = match step {
            Step::Sql(sql) => c.query(sql)?,
            Step::Copy(sql, data) => c.copy_in(sql, data.as_bytes())?,
        };
        let answer = slt::answer(None, &msgs);
        if answer.starts_with("error ") {
            let sql = match step {
                Step::Sql(s) | Step::Copy(s, _) => s,
            };
            let short: String = sql.chars().take(120).collect();
            return Ok(Some(format!("{short}: {answer}")));
        }
    }
    Ok(None)
}

/// The queries of a seed: rows, aggregates, groups and joins, with a predicate.
pub(crate) fn queries(rng: &mut Rng, tables: &[Table], count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            let a = &tables[rng.below(tables.len())];
            let p = logic::predicate(rng, &[a], DEPTH);
            let col = &a.cols[1 + rng.below(a.cols.len() - 1)].0;
            match rng.below(5) {
                0 => format!("SELECT * FROM {} WHERE {p}", a.name),
                1 => format!(
                    "SELECT count(*), count({col}), min({col}), max({col}), sum(id) FROM {} WHERE {p}",
                    a.name
                ),
                2 => format!("SELECT {col}, count(*), min(id) FROM {} WHERE {p} GROUP BY {col}", a.name),
                3 => format!("SELECT id FROM {} WHERE {p} ORDER BY id LIMIT 7", a.name),
                _ => {
                    let b = &tables[rng.below(tables.len())];
                    let on = logic::predicate(rng, &[a, b], 2);
                    let (x, y) = if a.name == b.name { ("x", "y") } else { (a.name.as_str(), b.name.as_str()) };
                    let on = if a.name == b.name {
                        on.replace(&format!("{}.", a.name), "x.")
                    } else {
                        on
                    };
                    let p = if a.name == b.name { p.replace(&format!("{}.", a.name), "x.") } else { p };
                    format!(
                        "SELECT count(*), sum({x}.id), sum({y}.id) FROM {} {x} JOIN {} {y} ON {x}.id = {y}.id AND {on} WHERE {p}",
                        a.name, b.name
                    )
                }
            }
        })
        .collect()
}

/// The count of the queries of one layout.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    pub(crate) layout: &'static str,
    /// The error of the setup, when the server does not accept the layout.
    pub(crate) missing: Option<String>,
    pub(crate) checked: usize,
    pub(crate) passed: usize,
    /// The queries whose answer is not the answer of the hot layout of the same server.
    pub(crate) not_hot: usize,
    pub(crate) skipped: usize,
}

/// A query whose answer in a layout is not the answer of the oracle.
#[derive(Clone, Debug)]
pub(crate) struct Fail {
    pub(crate) layout: &'static str,
    pub(crate) number: usize,
    pub(crate) sql: String,
    pub(crate) why: String,
}

/// The result of a run.
pub(crate) struct Outcome {
    pub(crate) data: Data,
    pub(crate) tallies: Vec<Tally>,
    pub(crate) fails: Vec<Fail>,
}

fn answer(c: &mut Conn, sql: &str) -> Result<String, String> {
    Ok(slt::answer(Some(Sort::Row), &c.query(sql)?))
}

/// Loads the data of the seed in each layout and runs `count` queries.
pub(crate) fn run(sides: &Sides, seed: u64, rows: usize, count: usize) -> Result<Outcome, String> {
    let mut rng = Rng::new(seed);
    let data = Data::new(&mut rng, rows);
    let list = queries(&mut rng, &data.tables, count);
    let long = Duration::from_secs(60);
    let mut base = generate::open(&sides.oracle, "layout_oracle", long, true)?;
    let mut changed = generate::open(&sides.oracle, "layout_oracle_changed", long, true)?;
    if let Some(e) = load(&mut base, &data.setup("hot only"))? {
        return Err(format!("the oracle does not load the data: {e}"));
    }
    let mut steps = data.setup("hot only");
    steps.extend(data.changes.iter().map(|c| Step::Sql(c.clone())));
    if let Some(e) = load(&mut changed, &steps)? {
        return Err(format!("the oracle does not load the changed data: {e}"));
    }
    let mut out = Outcome { data, tallies: Vec::new(), fails: Vec::new() };
    let mut conns: Vec<Option<Conn>> = Vec::new();
    for (i, &layout) in LAYOUTS.iter().enumerate() {
        let mut tally =
            Tally { layout, missing: None, checked: 0, passed: 0, not_hot: 0, skipped: 0 };
        let conn = generate::open(&sides.server, &format!("layout_{i}"), sides.timeout, true)
            .and_then(|mut c| {
                Ok(load(&mut c, &out.data.setup(layout))?.map_or(Some(c), |e| {
                    tally.missing = Some(e);
                    None
                }))
            });
        match conn {
            Ok(c) => conns.push(c),
            Err(e) => {
                tally.missing = Some(e);
                conns.push(None);
            }
        }
        out.tallies.push(tally);
    }
    for (n, sql) in list.iter().enumerate() {
        let want = answer(&mut base, sql)?;
        let want_changed = answer(&mut changed, sql)?;
        let mut hot = None;
        for (i, &layout) in LAYOUTS.iter().enumerate() {
            let Some(c) = &mut conns[i] else { continue };
            let after_change = layout == "mixed after change";
            let want = if after_change { &want_changed } else { &want };
            if want.starts_with("error ") {
                out.tallies[i].skipped += 1;
                continue;
            }
            let (got, lost) = match answer(c, sql) {
                Ok(a) => (a, false),
                Err(e) => (format!("no answer: {e}"), true),
            };
            let t = &mut out.tallies[i];
            t.checked += 1;
            if i == 0 {
                hot = Some(got.clone());
            } else if !after_change && hot.as_ref().is_some_and(|h| *h != got) {
                t.not_hot += 1;
            }
            if got == *want {
                t.passed += 1;
            } else {
                out.fails.push(Fail {
                    layout,
                    number: n + 1,
                    sql: sql.clone(),
                    why: format!("oracle: {want}\nserver: {got}"),
                });
            }
            if lost {
                conns[i] = None;
                out.tallies[i].missing = Some(format!("the server stops at query {}", n + 1));
            }
        }
    }
    drop((base, changed, conns));
    let mut c = Conn::connect(&sides.oracle, "template1")?;
    for db in ["layout_oracle", "layout_oracle_changed"] {
        c.execute(&format!("DROP DATABASE IF EXISTS {db} WITH (FORCE)"))?;
    }
    if let Ok(mut c) = Conn::connect(&sides.server, "template1") {
        for i in 0..LAYOUTS.len() {
            let _ = c.execute(&format!("DROP DATABASE IF EXISTS layout_{i} WITH (FORCE)"));
        }
    }
    Ok(out)
}

/// The SQL file of the failed queries: the data in the hot layout, the changes as a comment, then each query with the layout and the reason as a comment.
pub(crate) fn fail_file(seed: u64, out: &Outcome) -> String {
    let mut text = format!("-- rupg-compat layout --seed {seed}\n");
    for sql in out.data.sql() {
        text.push_str(&format!("{sql};\n"));
    }
    text.push_str("-- the changes of the layout \"mixed after change\":\n");
    for sql in &out.data.changes {
        text.push_str(&format!("-- {sql};\n"));
    }
    for f in &out.fails {
        text.push_str(&format!("\n-- {} {}\n", f.layout, f.number));
        for line in f.why.lines() {
            text.push_str(&format!("-- {line}\n"));
        }
        text.push_str(&format!("{};\n", f.sql));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_field_has_no_quotes() {
        assert_eq!(copy_field("NULL"), "\\N");
        assert_eq!(copy_field("'abc'"), "abc");
        assert_eq!(copy_field("''"), "");
        assert_eq!(copy_field("-2.25"), "-2.25");
    }

    #[test]
    fn each_layout_loads_every_row() {
        let data = Data::new(&mut Rng::new(7), 250);
        for layout in LAYOUTS {
            let mut rows = 0;
            for step in data.setup(layout) {
                match step {
                    Step::Sql(s) if s.starts_with("INSERT") && !data.changes.contains(&s) => {
                        rows += s.matches("), (").count() + 1;
                    }
                    Step::Copy(_, text) => rows += text.lines().count(),
                    _ => {}
                }
            }
            assert_eq!(rows, 3 * 250, "{layout}");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_queries() {
        let make = || {
            let mut rng = Rng::new(11);
            let data = Data::new(&mut rng, 20);
            (data.changes.clone(), queries(&mut rng, &data.tables, 30))
        };
        assert_eq!(make(), make());
    }

    #[test]
    fn a_join_of_a_table_with_itself_has_aliases() {
        let mut rng = Rng::new(3);
        let data = Data::new(&mut rng, 5);
        let mut seen = 0;
        for q in queries(&mut rng, &data.tables, 400).iter().filter(|q| q.contains(" x JOIN ")) {
            seen += 1;
            for t in &data.tables {
                assert!(!q.contains(&format!("{}.", t.name)), "{q}");
            }
        }
        assert!(seen > 0);
    }
}
