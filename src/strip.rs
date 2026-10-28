//! The summary deletion run of spec/21 section 21.8. A summary is a copy of a fact about the data, so rupg must give the same answers when every summary is gone.
//!
//! The run starts rupg on a new data directory and loads the data of a seed in the hot only, cold only, mixed and mixed after change layouts of `layout`, with a primary key and an index on each column. It runs the queries of the seed and keeps the answers. Then it stops rupg, copies the data directory, runs `rupg strip --summaries` on the copy and starts rupg on the copy. Each query must give the same answer, byte for byte. Then it drops each index that is not a constraint and compares again.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::bisect;
use crate::client::{Conn, Target};
use crate::generate::{self, Rng};
use crate::layout::{self, Data, Step};
use crate::oracle::{self, Oracle};
use crate::slt::{self, Sort};

/// The layouts of the run. The other layouts of `layout` change a setting of the session, not the file.
const LAYOUTS: [&str; 4] = ["hot only", "cold only", "mixed", "mixed after change"];
/// The comparisons, in the order of the report.
pub(crate) const CHECKS: [&str; 2] = ["stripped", "no indexes"];

/// The settings of a run.
pub(crate) struct Strip<'a> {
    /// The `rupg` binary of `rupg-cli`.
    pub(crate) bin: PathBuf,
    pub(crate) work: PathBuf,
    pub(crate) oracle: &'a Oracle,
    /// The server that the run starts, at `127.0.0.1` and the port of the run.
    pub(crate) target: Target,
    pub(crate) port: u16,
    pub(crate) timeout: Duration,
}

/// The count of one comparison.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    pub(crate) check: &'static str,
    /// The reason when the comparison did not run.
    pub(crate) missing: Option<String>,
    pub(crate) checked: usize,
    pub(crate) passed: usize,
}

/// A query whose answer after the change is not its answer on the original data directory.
#[derive(Clone, Debug)]
pub(crate) struct Fail {
    pub(crate) check: &'static str,
    pub(crate) layout: &'static str,
    pub(crate) number: usize,
    pub(crate) sql: String,
    pub(crate) why: String,
}

/// The result of a run.
pub(crate) struct Outcome {
    pub(crate) data: Data,
    /// The layouts that rupg does not accept, with the error.
    pub(crate) missing: Vec<(&'static str, String)>,
    pub(crate) tallies: Vec<Tally>,
    pub(crate) fails: Vec<Fail>,
}

impl Outcome {
    fn not_run(&mut self, why: &str) {
        for t in &mut self.tallies {
            t.missing.get_or_insert_with(|| why.to_string());
        }
    }
}

/// The primary key and the indexes of each table, and the names of the indexes that are not constraints.
fn indexes(data: &Data) -> (Vec<String>, Vec<String>) {
    let mut sql = Vec::new();
    let mut names = Vec::new();
    for t in &data.tables {
        sql.push(format!("ALTER TABLE {} ADD PRIMARY KEY (id)", t.name));
        for (c, _) in &t.cols[1..] {
            let name = format!("strip_{}_{c}", t.name);
            sql.push(format!("CREATE INDEX {name} ON {} ({c})", t.name));
            names.push(name);
        }
    }
    (sql, names)
}

/// Copies a directory and everything in it.
fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)
                .map_err(|e| format!("{}: {e}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn answers(c: &mut Conn, list: &[String]) -> Vec<String> {
    list.iter()
        .map(|sql| match c.query(sql) {
            Ok(msgs) => slt::answer(Some(Sort::Row), &msgs),
            Err(e) => format!("no answer: {e}"),
        })
        .collect()
}

impl Strip<'_> {
    /// Waits until the server takes a login, because a server can take a connection before it is ready (57P03).
    fn ready(&self) -> Result<(), String> {
        let clock = std::time::Instant::now();
        loop {
            match Conn::connect(&self.target, "postgres") {
                Ok(_) => return Ok(()),
                Err(e) if e.contains("57P03") && clock.elapsed() < Duration::from_secs(60) => {
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn dir(&self) -> PathBuf {
        self.work.join("strip")
    }

    /// Compares the answers of each layout with the answers on the original data directory.
    fn compare(
        &self,
        out: &mut Outcome,
        at: usize,
        want: &[Option<Vec<String>>],
        list: &[String],
    ) -> Result<(), String> {
        for (i, w) in want.iter().enumerate() {
            let Some(w) = w else { continue };
            let mut c = generate::open(&self.target, &format!("strip_{i}"), self.timeout, false)?;
            let got = answers(&mut c, list);
            for (n, (a, b)) in w.iter().zip(&got).enumerate() {
                out.tallies[at].checked += 1;
                if a == b {
                    out.tallies[at].passed += 1;
                } else {
                    out.fails.push(Fail {
                        check: CHECKS[at],
                        layout: LAYOUTS[i],
                        number: n + 1,
                        sql: list[n].clone(),
                        why: format!("original: {a}\nafter: {b}"),
                    });
                }
            }
        }
        Ok(())
    }

    /// Runs the comparisons for the data and the queries of a seed.
    pub(crate) fn run(&self, seed: u64, rows: usize, count: usize) -> Result<Outcome, String> {
        let mut rng = Rng::new(seed);
        let data = Data::new(&mut rng, rows);
        let list = layout::queries(&mut rng, &data.tables, count);
        let (index_sql, index_names) = indexes(&data);
        let mut out = Outcome {
            data,
            missing: Vec::new(),
            tallies: CHECKS
                .iter()
                .map(|&check| Tally { check, missing: None, checked: 0, passed: 0 })
                .collect(),
            fails: Vec::new(),
        };
        let dir = self.dir();
        let _ = fs::remove_dir_all(&dir);
        let (original, stripped) = (dir.join("original"), dir.join("stripped"));
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        self.oracle.write_conf(&original, self.port)?;
        let pw = dir.join("pw");
        fs::write(&pw, oracle::PASSWORD).map_err(|e| format!("{}: {e}", pw.display()))?;
        let server =
            bisect::start(&self.bin, &original, &pw, self.port, &dir.join("original.log"))??;
        if let Err(e) = self.ready() {
            bisect::stop(server);
            return Err(e);
        }
        let mut want: Vec<Option<Vec<String>>> = Vec::new();
        let mut loaded = Ok(());
        for (i, &name) in LAYOUTS.iter().enumerate() {
            let mut c =
                match generate::open(&self.target, &format!("strip_{i}"), self.timeout, true) {
                    Ok(c) => c,
                    Err(e) => {
                        loaded = Err(e);
                        break;
                    }
                };
            match layout::load(&mut c, &out.data.setup(name))? {
                Some(e) => {
                    out.missing.push((name, e));
                    want.push(None);
                }
                None => {
                    // An index that rupg cannot make is not an error of this run: the comparison without indexes still runs.
                    for sql in &index_sql {
                        let _ = layout::load(&mut c, &[Step::Sql(sql.clone())])?;
                    }
                    want.push(Some(answers(&mut c, &list)));
                }
            }
        }
        bisect::stop(server);
        loaded?;
        if want.iter().all(Option::is_none) {
            out.not_run("rupg accepts none of the layouts");
            return Ok(out);
        }
        copy_dir(&original, &stripped)?;
        let strip = Command::new(&self.bin)
            .arg("strip")
            .arg("--summaries")
            .arg(&stripped)
            .output()
            .map_err(|e| format!("{}: {e}", self.bin.display()))?;
        if !strip.status.success() {
            let text = String::from_utf8_lossy(&strip.stderr);
            let first = text.lines().next().unwrap_or_default();
            out.not_run(&format!("rupg strip --summaries gives {}: {first}", strip.status));
            return Ok(out);
        }
        let server =
            match bisect::start(&self.bin, &stripped, &pw, self.port, &dir.join("stripped.log"))? {
                Ok(s) => s,
                Err(e) => {
                    out.not_run(&format!("after rupg strip --summaries, {e}"));
                    return Ok(out);
                }
            };
        if let Err(e) = self.ready() {
            bisect::stop(server);
            return Err(e);
        }
        // A server that keeps its databases in memory has nothing in the copy.
        let first = want.iter().position(Option::is_some).unwrap_or(0);
        if let Err(e) = generate::open(&self.target, &format!("strip_{first}"), self.timeout, false)
        {
            bisect::stop(server);
            out.not_run(&format!("the copy of the data directory has no databases, so rupg serve does not keep them there ({e})"));
            return Ok(out);
        }
        let result = self.compare(&mut out, 0, &want, &list).and_then(|()| {
            for i in (0..want.len()).filter(|&i| want[i].is_some()) {
                let mut c =
                    generate::open(&self.target, &format!("strip_{i}"), self.timeout, false)?;
                for name in &index_names {
                    let _ = c.query(&format!("DROP INDEX IF EXISTS {name}"))?;
                }
            }
            self.compare(&mut out, 1, &want, &list)
        });
        bisect::stop(server);
        result?;
        Ok(out)
    }
}

/// The SQL file of the failed queries: the data in the hot layout, then each query with the comparison, the layout and the two answers as a comment.
pub(crate) fn fail_file(seed: u64, out: &Outcome) -> String {
    let mut text = format!("-- rupg-compat strip --seed {seed}\n");
    for sql in out.data.sql() {
        text.push_str(&format!("{sql};\n"));
    }
    for f in &out.fails {
        text.push_str(&format!("\n-- {}, {} {}\n", f.check, f.layout, f.number));
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
    fn each_column_but_the_key_has_an_index() {
        let data = Data::new(&mut Rng::new(5), 3);
        let (sql, names) = indexes(&data);
        let columns: usize = data.tables.iter().map(|t| t.cols.len() - 1).sum();
        assert_eq!(names.len(), columns);
        assert_eq!(sql.len(), columns + data.tables.len());
        assert!(sql.iter().filter(|s| s.contains("PRIMARY KEY (id)")).count() == data.tables.len());
    }

    #[test]
    fn a_copy_has_every_file() {
        let dir = std::env::temp_dir().join(format!("rupg-compat-strip-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("a/b")).unwrap();
        fs::write(dir.join("a/one"), "1").unwrap();
        fs::write(dir.join("a/b/two"), "2").unwrap();
        copy_dir(&dir.join("a"), &dir.join("c")).unwrap();
        assert_eq!(fs::read_to_string(dir.join("c/one")).unwrap(), "1");
        assert_eq!(fs::read_to_string(dir.join("c/b/two")).unwrap(), "2");
        fs::remove_dir_all(&dir).unwrap();
    }
}
