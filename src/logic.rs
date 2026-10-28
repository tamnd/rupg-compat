//! The logic oracles of spec/21 section 21.4.5: ternary logic partitioning (TLP), non-optimizing reference engine construction (NoREC) and pivoted query synthesis (PQS), with the differential oracle on each query.
//!
//! The seed makes the schema and the data of `generate`, and then a predicate over the columns of one or two tables for each check. The type of each expression is known, so that most predicates do not give an error. A check is first done on the oracle. When the relation is false on the oracle, or a query gives an error on the oracle, the check is skipped. Then the check is done on the server under test, and the answer of each query is compared with the answer of the oracle.
//!
//! - TLP: the rows of a query are the rows where the predicate is true, false or NULL, with `UNION ALL`. The same for `DISTINCT` with `UNION`, and for `count(*)`.
//! - NoREC: `count(*)` with the predicate in `WHERE` is the sum of `(p) IS TRUE` over all the rows, which the planner cannot use for an index or a filter.
//! - PQS: the oracle picks a row of a table and the value of the predicate on that row. The predicate is changed so that it is true on the row, and the row must be in the answer of the server.

use std::time::Duration;

use crate::client::Conn;
use crate::generate::{self, Rng, Sides, Table};
use crate::slt::{self, Sort};

/// The kinds of check, in the order of the report.
pub(crate) const KINDS: [&str; 4] = ["differential", "tlp", "norec", "pqs"];
/// The depth of a predicate.
const DEPTH: usize = 4;

/// The type of an expression.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ty {
    Int,
    Num,
    Text,
    Bool,
}

fn ty_of(sql: &str) -> Ty {
    match sql {
        "int4" | "int8" => Ty::Int,
        "numeric" | "float8" => Ty::Num,
        "text" => Ty::Text,
        _ => Ty::Bool,
    }
}

/// Makes typed expressions over the columns of the tables in `FROM`.
struct Expr<'a> {
    rng: &'a mut Rng,
    cols: Vec<(String, Ty)>,
}

impl Expr<'_> {
    fn new<'a>(rng: &'a mut Rng, tables: &[&Table]) -> Expr<'a> {
        let cols = tables
            .iter()
            .flat_map(|t| t.cols.iter().map(|(c, ty)| (format!("{}.{c}", t.name), ty_of(ty))))
            .collect();
        Expr { rng, cols }
    }

    fn any(&mut self) -> Ty {
        [Ty::Int, Ty::Num, Ty::Text, Ty::Bool][self.rng.below(4)]
    }

    fn leaf(&mut self, ty: Ty) -> String {
        let cols: Vec<&String> =
            self.cols.iter().filter(|(_, t)| *t == ty).map(|(c, _)| c).collect();
        if !cols.is_empty() && self.rng.below(3) != 0 {
            return cols[self.rng.below(cols.len())].clone();
        }
        if self.rng.below(8) == 0 {
            let cast = match ty {
                Ty::Int => "int4",
                Ty::Num => "numeric",
                Ty::Text => "text",
                Ty::Bool => "bool",
            };
            return format!("NULL::{cast}");
        }
        let list: &[&str] = match ty {
            Ty::Int => &["0", "1", "-1", "2", "7", "100"],
            Ty::Num => &["0.5", "-2.25", "1e3", "0.1", "1.5"],
            Ty::Text => &["'a'", "''", "'b'", "'abc'", "'A'"],
            Ty::Bool => &["TRUE", "FALSE"],
        };
        self.rng.pick(list).to_string()
    }

    fn expr(&mut self, ty: Ty, depth: usize) -> String {
        if depth == 0 || self.rng.below(4) == 0 {
            return self.leaf(ty);
        }
        let d = depth - 1;
        match ty {
            Ty::Bool => match self.rng.below(11) {
                0 => format!("({} AND {})", self.expr(Ty::Bool, d), self.expr(Ty::Bool, d)),
                1 => format!("({} OR {})", self.expr(Ty::Bool, d), self.expr(Ty::Bool, d)),
                2 => format!("(NOT {})", self.expr(Ty::Bool, d)),
                3 | 4 => {
                    let t = self.any();
                    let op = self.rng.pick(&["=", "<>", "<", "<=", ">", ">="]);
                    format!("({} {op} {})", self.expr(t, d), self.expr(t, d))
                }
                5 => {
                    let t = self.any();
                    let not = self.rng.pick(&["", "NOT "]);
                    format!("({} IS {not}NULL)", self.expr(t, d))
                }
                6 => {
                    let t = self.any();
                    let not = self.rng.pick(&["", "NOT "]);
                    format!("({} IS {not}DISTINCT FROM {})", self.expr(t, d), self.expr(t, d))
                }
                7 => {
                    let t = [Ty::Int, Ty::Num, Ty::Text][self.rng.below(3)];
                    let not = self.rng.pick(&["", "NOT "]);
                    let (a, b, c) = (self.expr(t, d), self.expr(t, d), self.expr(t, d));
                    format!("({a} {not}BETWEEN {b} AND {c})")
                }
                8 => {
                    let t = self.any();
                    let not = self.rng.pick(&["", "NOT "]);
                    let (a, b, c) = (self.expr(t, d), self.expr(t, d), self.expr(t, d));
                    format!("({a} {not}IN ({b}, {c}))")
                }
                9 => {
                    let pattern = self.rng.pick(&["'a%'", "'%b%'", "'_'", "''", "'%'"]);
                    let op = self.rng.pick(&["LIKE", "NOT LIKE", "ILIKE"]);
                    format!("({} {op} {pattern})", self.expr(Ty::Text, d))
                }
                _ => self.case(ty, d),
            },
            Ty::Int | Ty::Num => match self.rng.below(7) {
                0 => format!("({} + {})", self.expr(ty, d), self.expr(ty, d)),
                1 => format!("({} - {})", self.expr(ty, d), self.expr(ty, d)),
                2 => format!("({} * {})", self.expr(ty, d), self.leaf(ty)),
                3 => format!("abs({})", self.expr(ty, d)),
                4 => format!("COALESCE({}, {})", self.expr(ty, d), self.expr(ty, d)),
                5 => format!("NULLIF({}, {})", self.expr(ty, d), self.expr(ty, d)),
                _ => self.case(ty, d),
            },
            Ty::Text => match self.rng.below(6) {
                0 => format!("({} || {})", self.expr(ty, d), self.expr(ty, d)),
                1 => format!("lower({})", self.expr(ty, d)),
                2 => format!("upper({})", self.expr(ty, d)),
                3 => format!("COALESCE({}, {})", self.expr(ty, d), self.expr(ty, d)),
                4 => format!("substr({}, 1, 2)", self.expr(ty, d)),
                _ => self.case(ty, d),
            },
        }
    }

    fn case(&mut self, ty: Ty, d: usize) -> String {
        let (w, a, b) = (self.expr(Ty::Bool, d), self.expr(ty, d), self.expr(ty, d));
        format!("(CASE WHEN {w} THEN {a} ELSE {b} END)")
    }
}

/// One check: the kind, the queries and, for PQS, the row that must be in the answer.
#[derive(Clone, Debug)]
struct Check {
    kind: &'static str,
    first: String,
    second: String,
    pivot: Option<String>,
}

/// The count of the checks of one kind.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    pub(crate) kind: &'static str,
    pub(crate) checked: usize,
    pub(crate) passed: usize,
    pub(crate) skipped: usize,
}

/// A check that fails on the server under test.
#[derive(Clone, Debug)]
pub(crate) struct Fail {
    pub(crate) kind: &'static str,
    pub(crate) number: usize,
    pub(crate) first: String,
    pub(crate) second: String,
    pub(crate) why: String,
}

/// The result of a run.
#[derive(Clone, Debug)]
pub(crate) struct Outcome {
    pub(crate) schema: Vec<String>,
    pub(crate) tallies: Vec<Tally>,
    pub(crate) fails: Vec<Fail>,
}

/// The rows of an answer, without the line of the count. An error has no rows.
fn rows(answer: &str) -> Vec<&str> {
    if answer.starts_with("error ") {
        return Vec::new();
    }
    answer.lines().skip(1).collect()
}

/// A predicate of the given depth over the columns of the tables.
pub(crate) fn predicate(rng: &mut Rng, tables: &[&Table], depth: usize) -> String {
    Expr::new(rng, tables).expr(Ty::Bool, depth)
}

/// The checks of one predicate. The PQS check needs the oracle, to get the value of the predicate on a row.
fn checks(rng: &mut Rng, tables: &[Table], oracle: &mut Conn) -> Result<Vec<Check>, String> {
    let a = &tables[rng.below(tables.len())];
    let b = &tables[rng.below(tables.len())];
    let two = a.name != b.name && rng.below(2) == 0;
    let in_from: Vec<&Table> = if two { vec![a, b] } else { vec![a] };
    let mut e = Expr::new(rng, &in_from);
    let p = e.expr(Ty::Bool, DEPTH);
    let from = if two {
        let join = e.rng.pick(&["JOIN", "LEFT JOIN", "RIGHT JOIN", "FULL JOIN"]);
        let on = e.expr(Ty::Bool, 2);
        format!("{} {join} {} ON {on}", a.name, b.name)
    } else {
        a.name.clone()
    };
    let parts = |select: &str, all: &str| {
        [format!("({p})"), format!("(NOT ({p}))"), format!("(({p}) IS NULL)")]
            .iter()
            .map(|w| format!("SELECT {select} FROM {from} WHERE {w}"))
            .collect::<Vec<_>>()
            .join(&format!(" UNION {all}"))
    };
    let mut out = vec![
        Check {
            kind: "tlp",
            first: format!("SELECT * FROM {from}"),
            second: parts("*", "ALL "),
            pivot: None,
        },
        Check {
            kind: "tlp",
            first: format!("SELECT DISTINCT * FROM {from}"),
            second: parts("*", ""),
            pivot: None,
        },
        Check {
            kind: "tlp",
            first: format!("SELECT count(*) FROM {from}"),
            second: format!("SELECT sum(n) FROM ({}) AS s", parts("count(*) AS n", "ALL ")),
            pivot: None,
        },
        Check {
            kind: "norec",
            first: format!("SELECT count(*) FROM {from} WHERE {p}"),
            second: format!("SELECT COALESCE(sum((({p}) IS TRUE)::int4), 0) FROM {from}"),
            pivot: None,
        },
    ];
    // PQS on the first table: the value of the predicate on each row comes from the oracle.
    let mut e = Expr::new(rng, &[a]);
    let q = e.expr(Ty::Bool, DEPTH);
    let values =
        slt::answer(Some(Sort::Row), &oracle.query(&format!("SELECT *, ({q}) FROM {}", a.name))?);
    let all = rows(&values);
    if !all.is_empty() {
        let row = all[rng.below(all.len())];
        if let Some((pivot, value)) = row.rsplit_once(' ') {
            let fixed = match value {
                "\"t\"" => format!("({q})"),
                "\"f\"" => format!("(NOT ({q}))"),
                _ => format!("(({q}) IS NULL)"),
            };
            out.push(Check {
                kind: "pqs",
                first: format!("SELECT * FROM {} WHERE {fixed}", a.name),
                second: String::new(),
                pivot: Some(pivot.to_string()),
            });
        }
    }
    Ok(out)
}

/// Does the check on one side. It returns the answer of the first query, and the reason when the relation is false.
fn check(c: &mut Conn, k: &Check) -> Result<(String, Option<String>), String> {
    let first = slt::answer(Some(Sort::Row), &c.query(&k.first)?);
    if first.starts_with("error ") {
        return Ok((first.clone(), Some(format!("the query gives {first}"))));
    }
    if let Some(pivot) = &k.pivot {
        let why = (!rows(&first).contains(&pivot.as_str()))
            .then(|| format!("the row {pivot} is not in the answer"));
        return Ok((first, why));
    }
    let second = slt::answer(Some(Sort::Row), &c.query(&k.second)?);
    let why = (first != second).then(|| format!("first: {first}\nsecond: {second}"));
    Ok((first, why))
}

/// Runs the checks of `count` predicates of the seed.
pub(crate) fn run(sides: &Sides, seed: u64, count: usize) -> Result<Outcome, String> {
    let mut rng = Rng::new(seed);
    let (tables, schema) = generate::schema(&mut rng);
    let same = sides.oracle.addr == sides.server.addr;
    let server_db = if same { "logic_server" } else { "logic" };
    let mut o = generate::open(&sides.oracle, "logic_oracle", Duration::from_secs(60), true)?;
    let mut s = generate::open(&sides.server, server_db, sides.timeout, true);
    for sql in &schema {
        o.execute(sql)?;
        if let Ok(c) = &mut s {
            c.execute(sql)?;
        }
    }
    let mut out = Outcome {
        schema,
        tallies: KINDS
            .iter()
            .map(|&kind| Tally { kind, checked: 0, passed: 0, skipped: 0 })
            .collect(),
        fails: Vec::new(),
    };
    for n in 1..=count {
        for k in checks(&mut rng, &tables, &mut o)? {
            let at = KINDS.iter().position(|&kind| kind == k.kind).unwrap_or(0);
            let (want, why) = check(&mut o, &k)?;
            if why.is_some() {
                out.tallies[at].skipped += 1;
                out.tallies[0].skipped += 1;
                continue;
            }
            let got = match &mut s {
                Ok(c) => check(c, &k),
                Err(e) => Err(e.clone()),
            };
            let (got, why) = got.unwrap_or_else(|e| {
                s = generate::open(&sides.server, server_db, sides.timeout, false);
                (format!("no answer: {e}"), Some(format!("no answer: {e}")))
            });
            out.tallies[at].checked += 1;
            out.tallies[0].checked += 1;
            let mut fail = |kind: &'static str, why: String| {
                out.fails.push(Fail {
                    kind,
                    number: n,
                    first: k.first.clone(),
                    second: k.second.clone(),
                    why,
                });
            };
            match why {
                Some(why) => fail(k.kind, why),
                None => out.tallies[at].passed += 1,
            }
            if want == got {
                out.tallies[0].passed += 1;
            } else {
                fail("differential", format!("oracle: {want}\nserver: {got}"));
            }
        }
    }
    drop((o, s));
    let mut c = Conn::connect(&sides.oracle, "template1")?;
    c.execute("DROP DATABASE IF EXISTS logic_oracle WITH (FORCE)")?;
    if let Ok(mut c) = Conn::connect(&sides.server, "template1") {
        let _ = c.execute(&format!("DROP DATABASE IF EXISTS {server_db} WITH (FORCE)"));
    }
    Ok(out)
}

/// The SQL file of the failed checks: the schema, then the queries of each check with the reason as a comment.
pub(crate) fn fail_file(seed: u64, out: &Outcome) -> String {
    let mut text = format!("-- rupg-compat logic --seed {seed}\n");
    for sql in &out.schema {
        text.push_str(&format!("{sql};\n"));
    }
    for f in &out.fails {
        text.push_str(&format!(
            "\n-- {} {}\n-- {}\n{};\n",
            f.kind,
            f.number,
            f.why.replace('\n', "\n-- "),
            f.first
        ));
        if !f.second.is_empty() {
            text.push_str(&format!("{};\n", f.second));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables() -> Vec<Table> {
        vec![
            Table { name: "t0".into(), cols: vec![("c0".into(), "int4"), ("c1".into(), "text")] },
            Table { name: "t1".into(), cols: vec![("c1".into(), "bool"), ("c2".into(), "float8")] },
        ]
    }

    #[test]
    fn a_column_has_the_name_of_its_table() {
        let t = tables();
        let mut rng = Rng::new(3);
        let mut e = Expr::new(&mut rng, &[&t[0], &t[1]]);
        let names: Vec<&str> = e.cols.iter().map(|(c, _)| c.as_str()).collect();
        assert_eq!(names, ["t0.c0", "t0.c1", "t1.c1", "t1.c2"]);
        for _ in 0..50 {
            let p = e.expr(Ty::Bool, DEPTH);
            assert!(!p.contains(" c"), "{p}");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_predicates() {
        let t = tables();
        let make = |seed| {
            let mut rng = Rng::new(seed);
            let mut e = Expr::new(&mut rng, &[&t[0]]);
            (0..20).map(|_| e.expr(Ty::Bool, DEPTH)).collect::<Vec<_>>()
        };
        assert_eq!(make(5), make(5));
        assert_ne!(make(5), make(6));
    }

    #[test]
    fn rows_skip_the_count_and_errors() {
        assert_eq!(rows("2 lines\n\"1\" NULL\n\"2\" \"a\""), ["\"1\" NULL", "\"2\" \"a\""]);
        assert!(rows("error ERROR 42601 syntax error").is_empty());
        assert!(rows("0 lines\n").is_empty());
    }
}
