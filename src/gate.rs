//! Gate G1: seven clients show the gate schema (spec/21 section 21.4.2).
//!
//! The runner makes a new database on each server and runs `clients/gate.sql` in it with the psql of the oracle build. Then it runs `clients/<name>/run.sh gate` for each client. That program lists the tables, the columns, the types, the indexes and the constraints of the schema `gate` with the describe command or the object browser of the client, and writes the list to its standard output. The client passes when the list on the server under test is equal to the list on the oracle.
//!
//! Without `--server`, the runner checks the oracle against itself, and a client passes when its program exits with 0.
//!
//! The test of a client is `gate` in its lists of `src/fails.rs`. A test in `oracle-fail.txt` is not run. A test in `expected-fail.txt` still counts as a failure in the result file, but the run is good when it fails. A run is not good when a test fails that is not in `expected-fail.txt`, or when a test in that list passes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::client;
use crate::fails;
use crate::oracle::{self, Oracle};
use crate::suites::{self, Run, Server};

/// The gate schema.
pub(crate) const SCHEMA: &str = include_str!("../clients/gate.sql");
/// The database that the runner makes new on each server.
pub(crate) const DATABASE: &str = "compat_gate";
/// The seven clients of the gate, in the order of spec/21 section 21.4.2.
pub(crate) const CLIENTS: [&str; 7] =
    ["psql", "pgadmin4", "pgjdbc", "psycopg", "node-postgres", "pgx", "asyncpg"];
/// The gate as a suite of client programs.
pub(crate) const SUITE: Suite =
    Suite { name: "gate", database: DATABASE, schema: Some(SCHEMA), tls: false };
/// The time that one client program can take, with the install of the client.
const LIMIT: Duration = Duration::from_secs(900);

/// None when the two lists are equal, else the first line that differs.
pub(crate) fn difference(want: &str, got: &str) -> Option<String> {
    let (want, got): (Vec<&str>, Vec<&str>) = (want.lines().collect(), got.lines().collect());
    let n = want.iter().zip(&got).take_while(|(a, b)| a == b).count();
    if n == want.len() && n == got.len() {
        return None;
    }
    let line = |lines: &[&str]| lines.get(n).map_or("the end".into(), |l| format!("{l:?}"));
    Some(format!("line {}: want {}, got {}", n + 1, line(&want), line(&got)))
}

/// A suite of client programs: gate G1 or the connect suite. Each client has the program `clients/<name>/run.sh <suite>`, and the name of the suite is also the name of the test in the lists of the client.
#[derive(Debug)]
pub(crate) struct Suite {
    pub(crate) name: &'static str,
    /// The database of the programs. When the suite has a schema, the runner makes the database new on each server and runs the schema in it.
    pub(crate) database: &'static str,
    pub(crate) schema: Option<&'static str>,
    /// When set, the programs must connect with TLS and check the certificate of the oracle in `PGSSLROOTCERT`. libpq also gets `PGCHANNELBINDING=require`.
    pub(crate) tls: bool,
}

/// The output of one client program on one server.
struct Listing {
    passed: bool,
    text: String,
}

/// The state of one run.
struct Runner<'a> {
    o: &'a Oracle,
    suite: &'a Suite,
    /// The work directory, where the client programs keep their installs.
    work: PathBuf,
    out: PathBuf,
    log: String,
}

impl Runner<'_> {
    /// Runs a command with the environment of a server, logs it and stops it after `LIMIT`. The result is the exit status and the standard output.
    fn run(&mut self, server: &Server, what: &str, mut cmd: Command) -> Result<Listing, String> {
        server.env(self.o, &mut cmd);
        cmd.env("PGDATABASE", self.suite.database)
            .env("TZ", "UTC")
            .env("RUPG_COMPAT_WORK", &self.work);
        if self.suite.tls {
            cmd.env("PGSSLMODE", "verify-full")
                .env("PGSSLROOTCERT", self.out.join("root.crt"))
                .env("PGCHANNELBINDING", "require");
        }
        if let Some(bin) = self.o.bin("psql").parent() {
            cmd.env("RUPG_COMPAT_BIN", bin);
        }
        let stdout = self.out.join(format!("{}-stdout.txt", server.side()));
        let stderr = self.out.join(format!("{}-stderr.txt", server.side()));
        let file = |p: &Path| fs::File::create(p).map_err(|e| format!("{}: {e}", p.display()));
        cmd.stdin(Stdio::null()).stdout(file(&stdout)?).stderr(file(&stderr)?);
        let started = Instant::now();
        let mut child = cmd.spawn().map_err(|e| format!("{what}: {e}"))?;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| format!("{what}: {e}"))? {
                break status.to_string();
            }
            if started.elapsed() > LIMIT {
                let _ = child.kill();
                let _ = child.wait();
                break format!("no exit after {} s", LIMIT.as_secs());
            }
            thread::sleep(Duration::from_millis(100));
        };
        let passed = status == "exit status: 0";
        let text = fs::read_to_string(&stdout).unwrap_or_default();
        let errors = fs::read_to_string(&stderr).unwrap_or_default();
        self.log.push_str(&format!("$ {what} ({})\n{status}\n{errors}\n", server.side()));
        Ok(Listing { passed, text })
    }

    /// Makes the database of the suite on a server and runs the schema in it.
    fn schema(&mut self, server: &Server) -> Result<bool, String> {
        client::reset_database(&server.target(self.o), self.suite.database)?;
        let mut cmd = Command::new(self.o.bin("psql"));
        let file = self.out.join("schema.sql");
        cmd.args(["-X", "-q", "-v", "ON_ERROR_STOP=1", "-f"]).arg(&file);
        Ok(self.run(server, "psql -f schema.sql", cmd)?.passed)
    }

    /// Runs the program of the suite for a client on a server and keeps its output in `<side>-<client>.txt`.
    fn client(&mut self, server: &Server, name: &str) -> Result<Listing, String> {
        let suite = self.suite.name;
        let mut cmd = Command::new("sh");
        cmd.arg(Path::new("clients").join(name).join("run.sh")).arg(suite);
        let listing = self.run(server, &format!("clients/{name}/run.sh {suite}"), cmd)?;
        let path = self.out.join(format!("{}-{name}.txt", server.side()));
        fs::write(&path, &listing.text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(listing)
    }
}

/// Runs `suite` on `server` for the clients in `clients`. The flag is true when each result agrees with the lists of the clients.
pub(crate) fn run(
    o: &Oracle,
    suite: &Suite,
    server: &Server,
    work: &Path,
    clients: &[&str],
) -> Result<(Run, bool), String> {
    let started = Instant::now();
    let oracle =
        Server { addr: std::net::SocketAddr::from(([127, 0, 0, 1], o.port())), oracle: true };
    let out = suites::run_dir(work, o.pin.major, suite.name, server);
    suites::fresh_dir(&out)?;
    if suite.tls {
        fs::write(out.join("root.crt"), oracle::SERVER_CRT).map_err(|e| e.to_string())?;
    }
    let mut r = Runner { o, suite, work: work.to_path_buf(), out, log: String::new() };
    let mut run = Run { suite: suite.name.into(), ..Run::default() };

    let mut made = true;
    if let Some(schema) = suite.schema {
        fs::write(r.out.join("schema.sql"), schema).map_err(|e| e.to_string())?;
        if !r.schema(&oracle)? {
            return Err(format!(
                "the schema of {} fails on the oracle, see {}",
                suite.name,
                r.out.join("log.txt").display()
            ));
        }
        made = server.oracle || r.schema(server)?;
        run.tests.push(("schema".into(), made));
    }
    let mut good = made;
    let test = suite.name;
    for &name in clients {
        let fails = fails::load(name)?;
        if fails.oracle.iter().any(|t| t == test) {
            out!("  {name}: not run, {test} is in oracle-fail.txt");
            continue;
        }
        let expected = fails.expected.get(test);
        let want = r.client(&oracle, name)?;
        if !want.passed {
            out!("  {name}: the {test} program fails on the oracle");
            run.tests.push((name.into(), false));
            good = false;
            continue;
        }
        if server.oracle {
            run.tests.push((name.into(), true));
            continue;
        }
        let got = r.client(server, name)?;
        let same = match difference(&want.text, &got.text) {
            None => true,
            Some(d) => {
                r.log.push_str(&format!("{name} differs: {d}\n\n"));
                out!("  {name}: {d}");
                false
            }
        };
        let passed = made && got.passed && same;
        match (passed, expected) {
            (true, Some(_)) => {
                out!("  {name}: passes, so remove {test} from clients/{name}/expected-fail.txt");
                good = false;
            }
            (false, Some(cause)) => out!("  {name}: fails as expected-fail.txt gives: {cause}"),
            (false, None) => good = false,
            (true, None) => {}
        }
        run.tests.push((name.into(), passed));
    }

    let log = r.out.join("log.txt");
    fs::write(&log, &r.log).map_err(|e| format!("{}: {e}", log.display()))?;
    run.seconds = started.elapsed().as_secs();
    Ok((run, good))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_different_line_is_the_difference() {
        assert_eq!(difference("a\nb\n", "a\nb\n"), None);
        assert_eq!(difference("a\nb\n", "a\nc\n").unwrap(), "line 2: want \"b\", got \"c\"");
        assert_eq!(difference("a\nb\n", "a\n").unwrap(), "line 2: want \"b\", got the end");
    }

    #[test]
    fn each_client_has_a_run_script() {
        for name in CLIENTS {
            let script =
                fs::read_to_string(Path::new("clients").join(name).join("run.sh")).unwrap();
            assert!(script.contains("gate"), "clients/{name}/run.sh has no gate program");
        }
    }

    #[test]
    fn each_client_has_its_lists() {
        for name in CLIENTS {
            fails::load(name).unwrap();
        }
    }
}
