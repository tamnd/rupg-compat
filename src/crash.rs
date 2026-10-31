//! The crash run: a committed transaction stays after a crash, and a transaction that did not commit is gone.
//!
//! The run starts rupg on a new data directory and makes rounds of transactions from a seed. Each transaction runs on the oracle first. When the oracle commits it, it runs on rupg, and the transaction commits or rolls back on both. At the end of a round, rupg sometimes also has a transaction that is open, or a `CHECKPOINT`. Then the run stops rupg with SIGKILL, or with SIGTERM in each fourth round, and starts it again on the same data directory. Each table, and the list of the tables and indexes, must give the same answer as the oracle, which did not stop.

use std::fs;
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

use crate::bisect;
use crate::client::{self, Conn, Target};
use crate::generate::{self, Rng};
use crate::message::Msg;
use crate::oracle::{self, Oracle};
use crate::slt::{self, Sort};

/// The database of the run, on the oracle and on rupg.
const DATABASE: &str = "crash";
/// The query that lists the tables and indexes of the run.
const LIST: &str = "SELECT c.relname, c.relkind, a.attname, a.attnum FROM pg_class c LEFT JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum > 0 AND NOT a.attisdropped WHERE c.relnamespace = 'public'::regnamespace AND c.relkind IN ('r', 'i') ORDER BY 1, 4";

/// One table of the workload.
#[derive(Clone, Debug)]
struct Table {
    name: String,
    /// The columns after `id`, `a` and `b`, which `ALTER TABLE ADD COLUMN` made.
    added: usize,
    /// The next `id` to insert.
    next: u64,
}

/// One transaction of the workload.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Txn {
    pub(crate) statements: Vec<String>,
    pub(crate) commit: bool,
}

/// What the workload knows before a transaction.
#[derive(Debug)]
pub(crate) struct Saved(Vec<Table>, usize);

/// The rounds of a seed. The workload knows its tables, so it does not depend on the answers.
pub(crate) struct Workload {
    rng: Rng,
    tables: Vec<Table>,
    rows: usize,
    indexes: usize,
}

impl Workload {
    pub(crate) fn new(seed: u64, rows: usize) -> Workload {
        Workload { rng: Rng::new(seed), tables: Vec::new(), rows: rows.max(1), indexes: 0 }
    }

    fn create(&mut self) -> String {
        let name = format!("t{}", self.tables.len());
        let sql = format!("CREATE TABLE {name} (id bigint PRIMARY KEY, a int, b text)");
        self.tables.push(Table { name, added: 0, next: 1 });
        sql
    }

    fn statement(&mut self) -> String {
        if self.tables.is_empty() || self.rng.below(30) == 0 && self.tables.len() < 6 {
            return self.create();
        }
        let at = self.rng.below(self.tables.len());
        let rows = self.rows;
        let pick = self.rng.below(100);
        let (m, r, k) = (2 + self.rng.below(7), self.rng.below(2), 1 + self.rng.below(1000));
        let table = &mut self.tables[at];
        let name = table.name.clone();
        match pick {
            0..40 => {
                let n = 1 + self.rng.below(5);
                let values: Vec<String> = (0..n)
                    .map(|i| {
                        let id = table.next + i as u64;
                        format!("({id}, {}, 'v{id}')", id % 97)
                    })
                    .collect();
                table.next += n as u64;
                format!("INSERT INTO {name} (id, a, b) VALUES {}", values.join(", "))
            }
            40..50 => {
                let n = 1 + self.rng.below(rows) as u64;
                let (lo, hi) = (table.next, table.next + n - 1);
                table.next += n;
                format!(
                    "INSERT INTO {name} (id, a, b) SELECT g, g % 89, repeat('x', (g % 50)::int) || g FROM generate_series({lo}, {hi}) g"
                )
            }
            50..70 => format!("UPDATE {name} SET a = a + {k}, b = b || '{k}' WHERE id % {m} = {r}"),
            70..85 => format!("DELETE FROM {name} WHERE id % {m} = {r} AND a % 3 = {}", k % 3),
            85..88 => format!("TRUNCATE {name}"),
            88..92 if table.added < 3 => {
                table.added += 1;
                format!("ALTER TABLE {name} ADD COLUMN c{} int DEFAULT {k}", table.added)
            }
            92..95 => {
                self.indexes += 1;
                format!("CREATE INDEX i{} ON {name} (a)", self.indexes)
            }
            95..98 => format!(
                "UPDATE {name} SET id = id + 1000000000 WHERE id % {m} = {r} AND id < 1000000000"
            ),
            _ => format!(
                "DELETE FROM {name} WHERE id IN (SELECT id FROM {name} ORDER BY id LIMIT {k})"
            ),
        }
    }

    /// The number of transactions of a round: from one to four.
    pub(crate) fn round(&mut self) -> usize {
        1 + self.rng.below(4)
    }

    /// The next transaction, which commits with odds of four in five. When it does not commit, give the saved state to `undo`.
    pub(crate) fn txn(&mut self) -> (Txn, Saved) {
        let saved = Saved(self.tables.clone(), self.indexes);
        let n = 1 + self.rng.below(5);
        let statements = (0..n).map(|_| self.statement()).collect();
        (Txn { statements, commit: self.rng.below(5) != 0 }, saved)
    }

    /// Forgets the tables and the rows of a transaction that did not commit.
    pub(crate) fn undo(&mut self, saved: Saved) {
        (self.tables, self.indexes) = (saved.0, saved.1);
    }

    /// The statements of a transaction that is open at the crash, or none.
    pub(crate) fn open(&mut self) -> Option<Vec<String>> {
        if self.rng.below(3) != 0 {
            return None;
        }
        let (txn, saved) = self.txn();
        self.undo(saved);
        Some(txn.statements)
    }

    pub(crate) fn checkpoint(&mut self) -> bool {
        self.rng.below(4) == 0
    }

    fn checks(&self) -> Vec<String> {
        let mut list = vec![LIST.to_owned()];
        list.extend(self.tables.iter().map(|t| format!("SELECT * FROM {} ORDER BY id", t.name)));
        list
    }
}

/// A table whose answer after the restart is not the answer of the oracle, or a statement that fails on rupg only.
#[derive(Clone, Debug)]
pub(crate) struct Fail {
    pub(crate) round: usize,
    pub(crate) sql: String,
    pub(crate) why: String,
}

/// The result of a run.
pub(crate) struct Outcome {
    /// For each round, true when each check of the round passed.
    pub(crate) rounds: Vec<bool>,
    pub(crate) checked: usize,
    pub(crate) passed: usize,
    /// The statements that the oracle committed, in order.
    pub(crate) committed: Vec<String>,
    pub(crate) fails: Vec<Fail>,
    /// The reason when the run could not go on.
    pub(crate) stopped: Option<String>,
}

/// The settings of a run.
pub(crate) struct Crash<'a> {
    /// The `rupg` binary of `rupg-cli`.
    pub(crate) bin: PathBuf,
    pub(crate) work: PathBuf,
    pub(crate) oracle: &'a Oracle,
    pub(crate) oracle_target: Target,
    /// The server that the run starts, at `127.0.0.1` and the port of the run.
    pub(crate) target: Target,
    pub(crate) port: u16,
    pub(crate) timeout: Duration,
}

fn error(msgs: &[Msg]) -> Option<String> {
    msgs.iter().find(|m| m.name() == "ErrorResponse").map(client::error_text)
}

impl Crash<'_> {
    fn dir(&self) -> PathBuf {
        self.work.join("crash")
    }

    fn start(&self, round: usize) -> Result<Result<Child, String>, String> {
        let dir = self.dir();
        let log = dir.join(format!("serve-{round}.log"));
        let server = bisect::start(&self.bin, &dir.join("data"), &dir.join("pw"), self.port, &log)?;
        let Ok(server) = server else { return Ok(server) };
        let clock = std::time::Instant::now();
        loop {
            match Conn::connect(&self.target, "postgres") {
                Ok(_) => return Ok(Ok(server)),
                Err(e) if e.contains("57P03") && clock.elapsed() < Duration::from_secs(120) => {
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(e) => {
                    bisect::stop(server);
                    return Ok(Err(format!("after the restart: {e}")));
                }
            }
        }
    }

    /// Runs one transaction on both sides. It returns false when the oracle does not commit it.
    fn txn(
        &self,
        o: &mut Conn,
        s: &mut Conn,
        txn: &Txn,
        round: usize,
        out: &mut Outcome,
    ) -> Result<bool, String> {
        o.execute("BEGIN")?;
        for sql in &txn.statements {
            if error(&o.query(sql)?).is_some() {
                o.execute("ROLLBACK")?;
                return Ok(false);
            }
        }
        s.execute("BEGIN")?;
        for sql in &txn.statements {
            if let Some(e) = error(&s.query(sql)?) {
                let _ = s.query("ROLLBACK")?;
                o.execute("ROLLBACK")?;
                out.fails.push(Fail { round, sql: sql.clone(), why: format!("rupg only: {e}") });
                return Ok(false);
            }
        }
        let end = if txn.commit { "COMMIT" } else { "ROLLBACK" };
        o.execute(end)?;
        if let Some(e) = error(&s.query(end)?) {
            out.fails.push(Fail { round, sql: end.to_owned(), why: format!("rupg only: {e}") });
        }
        if txn.commit {
            out.committed.extend(txn.statements.iter().cloned());
        }
        Ok(txn.commit)
    }

    /// Runs `rounds` rounds of the seed.
    pub(crate) fn run(&self, seed: u64, rounds: usize, rows: usize) -> Result<Outcome, String> {
        let mut w = Workload::new(seed, rows);
        let mut out = Outcome {
            rounds: Vec::new(),
            checked: 0,
            passed: 0,
            committed: Vec::new(),
            fails: Vec::new(),
            stopped: None,
        };
        let dir = self.dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        self.oracle.write_conf(&dir.join("data"), self.port)?;
        let pw = dir.join("pw");
        fs::write(&pw, oracle::PASSWORD).map_err(|e| format!("{}: {e}", pw.display()))?;
        let mut o = generate::open(&self.oracle_target, DATABASE, self.timeout, true)?;
        let mut server = self.start(0)??;
        let mut s = match generate::open(&self.target, DATABASE, self.timeout, true) {
            Ok(s) => s,
            Err(e) => {
                bisect::stop(server);
                return Err(e);
            }
        };
        for round in 1..=rounds {
            let fails = out.fails.len();
            for _ in 0..w.round() {
                let (txn, saved) = w.txn();
                match self.txn(&mut o, &mut s, &txn, round, &mut out) {
                    Ok(true) => {}
                    Ok(false) => w.undo(saved),
                    Err(e) => {
                        bisect::stop(server);
                        return Err(e);
                    }
                }
            }
            if w.checkpoint() {
                let _ = s.query("CHECKPOINT");
            }
            if let Some(open) = w.open() {
                let _ = s.query("BEGIN");
                for sql in &open {
                    let _ = s.query(sql);
                }
            }
            drop(s);
            if round % 4 == 0 {
                bisect::stop(server);
            } else {
                let mut child = server;
                let _ = child.kill();
                let _ = child.wait();
            }
            server = match self.start(round)? {
                Ok(server) => server,
                Err(e) => {
                    out.stopped = Some(format!("round {round}: {e}"));
                    out.rounds.push(false);
                    return Ok(out);
                }
            };
            s = match generate::open(&self.target, DATABASE, self.timeout, false) {
                Ok(s) => s,
                Err(e) => {
                    bisect::stop(server);
                    out.stopped = Some(format!("round {round}: the database is gone: {e}"));
                    out.rounds.push(false);
                    return Ok(out);
                }
            };
            for sql in w.checks() {
                let want = slt::answer(Some(Sort::Row), &o.query(&sql)?);
                let got = match s.query(&sql) {
                    Ok(msgs) => slt::answer(Some(Sort::Row), &msgs),
                    Err(e) => format!("no answer: {e}"),
                };
                out.checked += 1;
                if want == got {
                    out.passed += 1;
                } else {
                    let cut = |a: &str| a.chars().take(2000).collect::<String>();
                    let why = format!("oracle: {}\nrupg: {}", cut(&want), cut(&got));
                    out.fails.push(Fail { round, sql, why });
                }
            }
            out.rounds.push(out.fails.len() == fails);
        }
        drop(s);
        bisect::stop(server);
        Ok(out)
    }
}

/// The SQL file of a failed run: the committed statements, then each failure as a comment with its statement.
pub(crate) fn fail_file(seed: u64, out: &Outcome) -> String {
    let mut text = format!("-- rupg-compat crash --seed {seed}\n");
    for sql in &out.committed {
        text.push_str(&format!("{sql};\n"));
    }
    for f in &out.fails {
        text.push_str(&format!("\n-- round {}\n", f.round));
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
    fn a_seed_gives_the_same_rounds() {
        let (mut a, mut b) = (Workload::new(7, 100), Workload::new(7, 100));
        for _ in 0..20 {
            assert_eq!(a.round(), b.round());
            assert_eq!(a.txn().0, b.txn().0);
            assert_eq!(a.open(), b.open());
        }
        assert_eq!(a.checks(), b.checks());
    }

    #[test]
    fn the_first_statement_makes_a_table() {
        let mut w = Workload::new(1, 10);
        let first = w.txn().0.statements.remove(0);
        assert_eq!(first, "CREATE TABLE t0 (id bigint PRIMARY KEY, a int, b text)");
        assert_eq!(w.checks()[1], "SELECT * FROM t0 ORDER BY id");
    }

    #[test]
    fn an_open_transaction_does_not_change_the_tables() {
        let mut w = Workload::new(3, 10);
        for _ in 0..10 {
            let _ = w.txn();
        }
        let checks = w.checks();
        for _ in 0..50 {
            let _ = w.open();
        }
        assert_eq!(w.checks(), checks);
        let (_, saved) = w.txn();
        let _ = w.txn();
        w.undo(saved);
        assert_eq!(w.checks(), checks);
    }

    #[test]
    fn the_fail_file_has_the_committed_statements() {
        let out = Outcome {
            rounds: vec![false],
            checked: 1,
            passed: 0,
            committed: vec!["CREATE TABLE t0 (id bigint PRIMARY KEY, a int, b text)".into()],
            fails: vec![Fail {
                round: 1,
                sql: "SELECT 1".into(),
                why: "oracle: 1\nrupg: 2".into(),
            }],
            stopped: None,
        };
        let text = fail_file(9, &out);
        assert!(text.starts_with("-- rupg-compat crash --seed 9\nCREATE TABLE t0"));
        assert!(text.ends_with("-- round 1\n-- oracle: 1\n-- rupg: 2\nSELECT 1;\n"));
    }
}
