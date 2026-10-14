//! The `pg_dump` round trip (spec/21 section 21.4.1).
//!
//! `pg_dump` of the oracle build must dump a database of the server under test, and a restore of that dump into the oracle must dump to the same text. The runner makes the database with `clients/pg_dump/roundtrip.sql` and runs these checks:
//!
//! - `schema`: psql runs the script on the server under test.
//! - `dump`: pg_dump dumps the database of the server under test.
//! - `same as the oracle`: the dump is equal to the dump of the oracle after the same script.
//! - `restore`: psql restores the dump into a new database of the oracle.
//! - `round trip`: the dump of that database is equal to the dump of the server under test.
//! - `custom format`: the same round trip with `pg_dump -Fc` and `pg_restore`.
//! - `restore into the server`: psql restores the dump of the oracle into a new database of the server under test, and the dump of that database is equal to the dump of the oracle.
//!
//! The comparison ignores the line `-- Dumped from database version`, because it has the version of the server, and the lines `\restrict` and `\unrestrict`, because pg_dump makes a new random key for each dump.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::client;
use crate::oracle::Oracle;
use crate::suites::{self, Run, Server};

/// The schema of the round trip.
pub(crate) const SCHEMA: &str = include_str!("../clients/pg_dump/roundtrip.sql");
/// The database that the runner makes new for the schema on each server.
pub(crate) const DATABASE: &str = "compat_dump";
/// The database that the runner makes new for each restore.
pub(crate) const RESTORE_DATABASE: &str = "compat_restore";

/// The lines of a dump without the lines that change from one server or one run to the next.
pub(crate) fn normalize(dump: &str) -> Vec<&str> {
    dump.lines()
        .filter(|l| {
            !l.starts_with("-- Dumped from database version")
                && !l.starts_with("\\restrict ")
                && !l.starts_with("\\unrestrict ")
        })
        .collect()
}

/// None when the two dumps are equal after `normalize`, else the first line that differs.
pub(crate) fn difference(want: &str, got: &str) -> Option<String> {
    let (want, got) = (normalize(want), normalize(got));
    let n = want.iter().zip(&got).take_while(|(a, b)| a == b).count();
    if n == want.len() && n == got.len() {
        return None;
    }
    let line =
        |lines: &[&str]| lines.get(n).map_or("the end of the dump".into(), |l| format!("{l:?}"));
    Some(format!("line {} of the dump: want {}, got {}", n + 1, line(&want), line(&got)))
}

/// The state of one run: the oracle build, the directory of the files and the log.
struct Runner<'a> {
    o: &'a Oracle,
    out: PathBuf,
    log: String,
}

impl Runner<'_> {
    /// Runs a program of the oracle build against a server and logs its output. True when it exits with 0.
    fn program(&mut self, server: &Server, name: &str, args: &[&str]) -> Result<bool, String> {
        let path = self.o.bin(name);
        let mut cmd = Command::new(&path);
        cmd.args(args).stdin(Stdio::null());
        server.env(self.o, &mut cmd);
        let r = cmd.output().map_err(|e| format!("{}: {e}", path.display()))?;
        self.log.push_str(&format!(
            "$ {name} {} ({})\n{}\n{}{}\n",
            args.join(" "),
            server.side(),
            r.status,
            String::from_utf8_lossy(&r.stdout),
            String::from_utf8_lossy(&r.stderr)
        ));
        Ok(r.status.success())
    }

    fn file(&self, name: &str) -> String {
        self.out.join(name).display().to_string()
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.out.join(name)).unwrap_or_default()
    }

    /// Runs a script with psql in a database. True when each statement passes.
    fn psql(&mut self, server: &Server, database: &str, file: &str) -> Result<bool, String> {
        self.program(
            server,
            "psql",
            &["-X", "-q", "-v", "ON_ERROR_STOP=1", "-d", database, "-f", file],
        )
    }

    /// Dumps a database as text to a file of the run directory.
    fn dump(&mut self, server: &Server, database: &str, name: &str) -> Result<bool, String> {
        let file = self.file(name);
        self.program(server, "pg_dump", &["-d", database, "-f", &file])
    }

    /// Compares two dumps of the run directory and logs the first difference.
    fn same(&mut self, want: &str, got: &str) -> bool {
        match difference(&self.read(want), &self.read(got)) {
            None => true,
            Some(d) => {
                self.log.push_str(&format!("{got} differs from {want}: {d}\n\n"));
                out!("  {got} differs from {want}: {d}");
                false
            }
        }
    }
}

/// Runs the round trip between the oracle and `server`. Without `--server`, `server` is the oracle, and the run checks the oracle against itself.
pub(crate) fn round_trip(o: &Oracle, server: &Server, work: &Path) -> Result<Run, String> {
    let started = Instant::now();
    let oracle = Server { addr: oracle_addr(o), oracle: true };
    let out = suites::run_dir(work, o.pin.major, "dump", server);
    suites::fresh_dir(&out)?;
    fs::write(out.join("roundtrip.sql"), SCHEMA).map_err(|e| e.to_string())?;
    let mut r = Runner { o, out, log: String::new() };
    let schema = r.file("roundtrip.sql");
    let mut run = Run { suite: "dump".into(), ..Run::default() };

    client::reset_database(&server.target(o), DATABASE)?;
    let made = r.psql(server, DATABASE, &schema)?;
    run.tests.push(("schema".into(), made));
    let dumped = made && r.dump(server, DATABASE, "server.sql")?;
    run.tests.push(("dump".into(), dumped));

    client::reset_database(&oracle.target(o), DATABASE)?;
    let want = r.psql(&oracle, DATABASE, &schema)? && r.dump(&oracle, DATABASE, "oracle.sql")?;
    if !want {
        return Err(format!(
            "the schema of the round trip fails on the oracle, see {}",
            r.file("log.txt")
        ));
    }
    run.tests.push(("same as the oracle".into(), dumped && r.same("oracle.sql", "server.sql")));

    client::reset_database(&oracle.target(o), RESTORE_DATABASE)?;
    let restored = dumped && r.psql(&oracle, RESTORE_DATABASE, &r.file("server.sql"))?;
    run.tests.push(("restore".into(), restored));
    let again = restored && r.dump(&oracle, RESTORE_DATABASE, "restored.sql")?;
    run.tests.push(("round trip".into(), again && r.same("server.sql", "restored.sql")));

    let archive = r.file("server.dump");
    let custom = made && r.program(server, "pg_dump", &["-Fc", "-d", DATABASE, "-f", &archive])?;
    client::reset_database(&oracle.target(o), RESTORE_DATABASE)?;
    let custom = custom
        && r.program(
            &oracle,
            "pg_restore",
            &["--exit-on-error", "-d", RESTORE_DATABASE, &archive],
        )?
        && r.dump(&oracle, RESTORE_DATABASE, "restored-custom.sql")?;
    run.tests.push(("custom format".into(), custom && r.same("server.sql", "restored-custom.sql")));

    client::reset_database(&server.target(o), RESTORE_DATABASE)?;
    let back = r.psql(server, RESTORE_DATABASE, &r.file("oracle.sql"))?
        && r.dump(server, RESTORE_DATABASE, "server-restored.sql")?;
    run.tests.push((
        "restore into the server".into(),
        back && r.same("oracle.sql", "server-restored.sql"),
    ));

    let log = r.out.join("log.txt");
    fs::write(&log, &r.log).map_err(|e| format!("{}: {e}", log.display()))?;
    run.seconds = started.elapsed().as_secs();
    Ok(run)
}

/// The address of the oracle.
fn oracle_addr(o: &Oracle) -> std::net::SocketAddr {
    std::net::SocketAddr::from(([127, 0, 0, 1], o.port()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMP: &str = "\\restrict abc\n--\n-- Dumped from database version 19.0\n-- Dumped by pg_dump version 19.0\nCREATE TABLE t (a integer);\n\\unrestrict abc\n";

    #[test]
    fn the_version_and_the_keys_leave_the_comparison() {
        assert_eq!(
            normalize(DUMP),
            ["--", "-- Dumped by pg_dump version 19.0", "CREATE TABLE t (a integer);"]
        );
        let other = DUMP
            .replace("abc", "xyz")
            .replace("database version 19.0", "database version 19.0 (rupg)");
        assert_eq!(difference(DUMP, &other), None);
    }

    #[test]
    fn the_first_different_line_is_the_difference() {
        let other = DUMP.replace("a integer", "a bigint");
        assert_eq!(
            difference(DUMP, &other).unwrap(),
            "line 3 of the dump: want \"CREATE TABLE t (a integer);\", got \"CREATE TABLE t (a bigint);\""
        );
        let short = DUMP.replace("CREATE TABLE t (a integer);\n", "");
        assert_eq!(
            difference(DUMP, &short).unwrap(),
            "line 3 of the dump: want \"CREATE TABLE t (a integer);\", got the end of the dump"
        );
    }
}
