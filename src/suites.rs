//! The runners of the PostgreSQL suites: the regression suite, the isolation suite and `libpq_pipeline` (spec/21 section 21.4.1).
//!
//! Each runner uses the test program of the oracle build at the pin, checks the files of the suite against the manifest of `rupg-compat import`, and runs the suite against one server: the oracle or the server under test. It writes the pass set to `results/N/<suite>-<side>.toml` in the work directory, where the report reads it.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::client;
use crate::import::{self, ISOLATION, PIPELINE, REGRESS};
use crate::oracle::{self, Oracle};

/// The tests of `libpq_pipeline` whose libpq trace the TAP test compares with `traces/T.trace`.
pub(crate) const TRACED: [&str; 9] = [
    "simple_pipeline",
    "nosync",
    "multi_pipelines",
    "prepared",
    "singlerow",
    "pipeline_abort",
    "pipeline_idle",
    "transaction",
    "disallowed_in_pipeline",
];

/// The database that the runner makes new for each run of `libpq_pipeline`.
pub(crate) const PIPELINE_DATABASE: &str = "compat_pipeline";

/// The server that a suite runs against.
#[derive(Clone, Debug)]
pub(crate) struct Server {
    pub(crate) addr: SocketAddr,
    /// True for the oracle itself. Otherwise the server under test gets `rupg.compat_version`.
    pub(crate) oracle: bool,
}

impl Server {
    pub(crate) fn side(&self) -> &'static str {
        if self.oracle { "oracle" } else { "server" }
    }

    /// The target of the client of the harness, with `rupg.compat_version` for the server under test.
    fn target(&self, o: &Oracle) -> client::Target {
        let mut params = Vec::new();
        if !self.oracle {
            params.push(("rupg.compat_version".to_string(), o.pin.major.to_string()));
        }
        client::Target {
            addr: self.addr,
            user: oracle::USER.into(),
            password: oracle::PASSWORD.into(),
            params,
        }
    }

    /// The libpq environment of a test program. The suites run without TLS, as PostgreSQL runs them on a Unix socket.
    fn env(&self, o: &Oracle, cmd: &mut Command) {
        cmd.env("PGHOST", self.addr.ip().to_string())
            .env("PGPORT", self.addr.port().to_string())
            .env("PGUSER", oracle::USER)
            .env("PGPASSWORD", oracle::PASSWORD)
            .env("PGSSLMODE", "disable")
            .env_remove("PGDATABASE")
            .env_remove("PGSERVICE");
        if self.oracle {
            cmd.env_remove("PGOPTIONS");
        } else {
            cmd.env("PGOPTIONS", format!("-c rupg.compat_version={}", o.pin.major));
        }
    }
}

/// The result of one run of one suite.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) suite: String,
    pub(crate) tests: Vec<(String, bool)>,
    /// The cases that the unstable filter took out. They are in neither list.
    pub(crate) unstable: Vec<String>,
    pub(crate) seconds: u64,
}

impl Run {
    pub(crate) fn passed(&self) -> usize {
        self.tests.iter().filter(|t| t.1).count()
    }

    pub(crate) fn failed(&self) -> Vec<&str> {
        self.tests.iter().filter(|t| !t.1).map(|t| t.0.as_str()).collect()
    }

    /// The result file, in the subset of TOML that `src/toml.rs` reads.
    pub(crate) fn to_toml(&self, o: &Oracle, server: &Server, started: &str) -> String {
        let quote =
            |name: &str| format!("  \"{}\",\n", name.replace('\\', "\\\\").replace('"', "\\\""));
        let list = |pass: bool| {
            self.tests.iter().filter(|t| t.1 == pass).map(|t| quote(&t.0)).collect::<String>()
        };
        let unstable = self.unstable.iter().map(|n| quote(n)).collect::<String>();
        format!(
            "# Written by rupg-compat {suite}.\nsuite = \"{suite}\"\nversion = {}\ncommit = \"{}\"\nserver = \"{}\"\nside = \"{}\"\nstarted = \"{started}\"\nseconds = {}\ntotal = {}\npassed = {}\npass = [\n{}]\nfail = [\n{}]\nunstable = [\n{unstable}]\n",
            o.pin.major,
            o.pin.commit,
            server.addr,
            server.side(),
            self.seconds,
            self.tests.len(),
            self.passed(),
            list(true),
            list(false),
            suite = self.suite,
        )
    }
}

/// The pass or fail of each test in the output of `pg_regress` or `pg_isolation_regress`. Both the TAP lines of 16 and later (`ok 2 + boolean 418 ms`) and the older lines (`test boolean ... ok`) are read.
pub(crate) fn parse_results(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        let tap = t
            .strip_prefix("ok ")
            .map(|r| (r, true))
            .or_else(|| t.strip_prefix("not ok ").map(|r| (r, false)));
        if let Some((rest, pass)) = tap {
            let words: Vec<&str> = rest.split_whitespace().collect();
            if let [_, mark, name, ..] = words[..]
                && (mark == "+" || mark == "-")
            {
                out.push((name.to_string(), pass));
            }
        } else if let Some((before, after)) = t.split_once(" ... ")
            && let Some(name) = before.split_whitespace().last()
        {
            out.push((name.to_string(), after.trim_start().starts_with("ok")));
        }
    }
    out
}

/// The pass set of a schedule: each scheduled test passes when the output says ok. A test that the output does not name fails.
fn pass_set(scheduled: &[String], results: &[(String, bool)]) -> Vec<(String, bool)> {
    scheduled.iter().map(|t| (t.clone(), results.iter().any(|(n, ok)| n == t && *ok))).collect()
}

pub(crate) fn utc_now() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    utc(secs)
}

/// The UTC time of a Unix time, as `YYYY-MM-DDTHH:MM:SSZ`.
pub(crate) fn utc(secs: u64) -> String {
    let (days, rest) = (secs / 86_400, secs % 86_400);
    // The civil date of a day number, from the algorithm of Howard Hinnant.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rest / 3600, rest / 60 % 60, rest % 60)
}

/// The directory of the runs of one suite on one side.
pub(crate) fn run_dir(work: &Path, major: u32, suite: &str, server: &Server) -> PathBuf {
    work.join("runs").join(major.to_string()).join(format!("{suite}-{}", server.side()))
}

/// The result file of one suite on one side.
pub(crate) fn result_file(work: &Path, major: u32, suite: &str, side: &str) -> PathBuf {
    work.join("results").join(major.to_string()).join(format!("{suite}-{side}.toml"))
}

fn fresh_dir(dir: &Path) -> Result<(), String> {
    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))
}

/// Runs `pg_regress` or `pg_isolation_regress` of the oracle build once, with a schedule or with a list of tests. Returns the exit status and the output.
fn invoke(
    o: &Oracle,
    server: &Server,
    dir: &str,
    program: &str,
    tests: &[String],
    out: &Path,
) -> Result<(bool, String), String> {
    fresh_dir(out)?;
    let cwd = o.src().join(dir);
    let mut cmd = Command::new(cwd.join(program));
    cmd.current_dir(&cwd)
        .arg("--inputdir=.")
        .arg(format!("--bindir={}", o.install().join("bin").display()))
        .arg(format!("--outputdir={}", out.display()))
        .arg(format!("--host={}", server.addr.ip()))
        .arg(format!("--port={}", server.addr.port()))
        .arg(format!("--user={}", oracle::USER));
    if dir == REGRESS {
        cmd.arg("--dlpath=.").arg("--max-concurrent-tests=20");
        // In 14, the makefile asks pg_regress to make the directory of the tablespace test. From 15, pg_regress always makes it.
        if o.pin.major <= 14 {
            cmd.arg("--make-testtablespace-dir");
        }
    }
    cmd.args(tests).stdin(Stdio::null());
    server.env(o, &mut cmd);
    let file = out.join("stdout.txt");
    let log = fs::File::create(&file).map_err(|e| e.to_string())?;
    let status = cmd
        .stdout(log.try_clone().map_err(|e| e.to_string())?)
        .stderr(log)
        .status()
        .map_err(|e| format!("{program}: {e}"))?;
    let text = fs::read_to_string(&file).map_err(|e| e.to_string())?;
    Ok((status.success(), text))
}

/// A suite that runs with `pg_regress` or `pg_isolation_regress`.
struct Suite {
    name: &'static str,
    dir: &'static str,
    program: &'static str,
    schedule: &'static str,
}

const REGRESS_SUITE: Suite =
    Suite { name: "regress", dir: REGRESS, program: "pg_regress", schedule: "parallel_schedule" };
const ISOLATION_SUITE: Suite = Suite {
    name: "isolation",
    dir: ISOLATION,
    program: "pg_isolation_regress",
    schedule: "isolation_schedule",
};

/// Runs a suite: the schedule, then the tests of the suite that the schedule does not list.
fn run_suite(
    o: &Oracle,
    corpus: &Path,
    server: &Server,
    work: &Path,
    s: &Suite,
    unscheduled: &[String],
) -> Result<Run, String> {
    let checked = import::verify(&o.src(), &import::corpus_dir(corpus, o.pin.major), s.dir)?;
    out!("{} {}: {checked} files equal to the manifest of the pin", s.name, o.pin.major);
    let out = run_dir(work, o.pin.major, s.name, server);
    fresh_dir(&out)?;
    let cwd = o.src().join(s.dir);
    let mut scheduled = import::schedule_tests(
        &fs::read_to_string(cwd.join(s.schedule)).map_err(|e| e.to_string())?,
    );
    let started = Instant::now();
    let (ok, text) = invoke(
        o,
        server,
        s.dir,
        s.program,
        &[format!("--schedule=./{}", s.schedule)],
        &out.join("schedule"),
    )?;
    let mut results = parse_results(&text);
    let mut all_ok = ok;
    if !unscheduled.is_empty() {
        let (ok, text) =
            invoke(o, server, s.dir, s.program, unscheduled, &out.join("unscheduled"))?;
        results.extend(parse_results(&text));
        scheduled.extend(unscheduled.iter().cloned());
        all_ok &= ok;
    }
    let run = Run {
        suite: s.name.into(),
        tests: pass_set(&scheduled, &results),
        seconds: started.elapsed().as_secs(),
        ..Run::default()
    };
    if run.passed() == run.tests.len() && !all_ok {
        return Err(format!(
            "{} failed but every test passed; see the output in {}",
            s.program,
            out.display()
        ));
    }
    Ok(run)
}

pub(crate) fn regress(
    o: &Oracle,
    corpus: &Path,
    server: &Server,
    work: &Path,
) -> Result<Run, String> {
    run_suite(o, corpus, server, work, &REGRESS_SUITE, &[])
}

/// The specs of `specs/` that `isolation_schedule` does not list. In 19 they are `prepared-transactions` and `prepared-transactions-cic`, which PostgreSQL runs with `make installcheck-prepared-txns`.
pub(crate) fn unscheduled_specs(o: &Oracle) -> Result<Vec<String>, String> {
    let cwd = o.src().join(ISOLATION);
    let scheduled = import::schedule_tests(
        &fs::read_to_string(cwd.join("isolation_schedule")).map_err(|e| e.to_string())?,
    );
    let mut specs: Vec<String> = fs::read_dir(cwd.join("specs"))
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()?.file_name().to_str()?.strip_suffix(".spec").map(str::to_string))
        .filter(|s| !scheduled.contains(s))
        .collect();
    specs.sort();
    Ok(specs)
}

pub(crate) fn isolation(
    o: &Oracle,
    corpus: &Path,
    server: &Server,
    work: &Path,
) -> Result<Run, String> {
    let extra = unscheduled_specs(o)?;
    run_suite(o, corpus, server, work, &ISOLATION_SUITE, &extra)
}

/// Runs `libpq_pipeline` as its TAP test does: each test with 700 rows, and the libpq trace of the traced tests compared with `traces/T.trace`. The result has one row for each run and one row `T trace` for each traced test.
pub(crate) fn pipeline(
    o: &Oracle,
    corpus: &Path,
    server: &Server,
    work: &Path,
) -> Result<Run, String> {
    let checked = import::verify(&o.src(), &import::corpus_dir(corpus, o.pin.major), PIPELINE)?;
    out!("pipeline {}: {checked} files equal to the manifest of the pin", o.pin.major);
    let out = run_dir(work, o.pin.major, "pipeline", server);
    fresh_dir(&out)?;
    let dir = o.src().join(PIPELINE);
    let program = dir.join("libpq_pipeline");
    let list = Command::new(&program)
        .arg("tests")
        .output()
        .map_err(|e| format!("{}: {e}", program.display()))?;
    let tests: Vec<String> =
        String::from_utf8_lossy(&list.stdout).split_whitespace().map(str::to_string).collect();
    // The TAP test runs on a new cluster. The runner gives the tests a new database, so the tables of an earlier run do not change the notices of `DROP TABLE IF EXISTS` in the traces.
    client::reset_database(&server.target(o), PIPELINE_DATABASE)?;
    let conninfo = format!(
        "host={} port={} user={} password={} dbname={PIPELINE_DATABASE} sslmode=disable",
        server.addr.ip(),
        server.addr.port(),
        oracle::USER,
        oracle::PASSWORD
    );
    let latest = if o.pin.major >= 18 { " max_protocol_version=latest" } else { "" };
    let started = Instant::now();
    let mut run = Run { suite: "pipeline".into(), ..Run::default() };
    let mut log = String::new();
    let mut one = |name: String, args: Vec<String>, run: &mut Run| -> Result<bool, String> {
        let mut cmd = Command::new(&program);
        cmd.args(&args).stdin(Stdio::null());
        server.env(o, &mut cmd);
        let r = cmd.output().map_err(|e| e.to_string())?;
        log.push_str(&format!(
            "$ libpq_pipeline {}\nexit {}\n{}{}\n",
            args.join(" "),
            r.status,
            String::from_utf8_lossy(&r.stdout),
            String::from_utf8_lossy(&r.stderr)
        ));
        run.tests.push((name, r.status.success()));
        Ok(r.status.success())
    };
    for t in &tests {
        let traced = TRACED.contains(&t.as_str());
        let trace = out.join(format!("{t}.trace"));
        let mut args = vec!["-r".to_string(), "700".into()];
        if traced {
            args.extend(["-t".into(), trace.display().to_string()]);
        }
        args.extend([t.clone(), format!("{conninfo}{latest}")]);
        one(t.clone(), args, &mut run)?;
        if traced {
            let want = fs::read(dir.join("traces").join(format!("{t}.trace"))).unwrap_or_default();
            let got = fs::read(&trace).unwrap_or_default();
            run.tests.push((format!("{t} trace"), !want.is_empty() && want == got));
        }
    }
    if o.pin.major >= 18 {
        // From 18, the TAP test also runs the cancel test with protocol 3.0.
        let args = vec!["cancel".to_string(), format!("{conninfo} max_protocol_version=3.0")];
        one("cancel (protocol 3.0)".into(), args, &mut run)?;
    }
    fs::write(out.join("stdout.txt"), log).map_err(|e| e.to_string())?;
    run.seconds = started.elapsed().as_secs();
    Ok(run)
}

/// Writes the result file and prints the summary of a run.
pub(crate) fn report_run(
    run: &Run,
    o: &Oracle,
    server: &Server,
    work: &Path,
    started: &str,
) -> Result<PathBuf, String> {
    let path = result_file(work, o.pin.major, &run.suite, server.side());
    fs::create_dir_all(path.parent().unwrap_or(work)).map_err(|e| e.to_string())?;
    fs::write(&path, run.to_toml(o, server, started))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    out!(
        "{} {} on {} ({}): {} of {} pass in {} s; result in {}",
        run.suite,
        o.pin.major,
        server.addr,
        server.side(),
        run.passed(),
        run.tests.len(),
        run.seconds,
        path.display()
    );
    if !run.unstable.is_empty() {
        out!(
            "  {} unstable on the oracle, not counted: {}",
            run.unstable.len(),
            run.unstable.join(", ")
        );
    }
    for f in run.failed() {
        out!("  fail: {f}");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tap_lines_of_16_and_later_are_read() {
        let text = "# using postmaster on 127.0.0.1, port 54319\nok 1         - test_setup                               1082 ms\n# parallel group (2 tests):  int4 boolean\nok 2         + boolean                                   418 ms\nnot ok 3     + int4                                      332 ms\n1..3\n";
        assert_eq!(
            parse_results(text),
            [("test_setup".to_string(), true), ("boolean".into(), true), ("int4".into(), false)]
        );
    }

    #[test]
    fn the_lines_of_14_and_15_are_read() {
        let text = "test tablespace                   ... ok          512 ms\nparallel group (2 tests):  boolean char\n     boolean                      ... FAILED      100 ms\n     char                         ... failed (ignored)       90 ms\n     name                         ... ok           20 ms\n";
        assert_eq!(
            parse_results(text),
            [
                ("tablespace".to_string(), true),
                ("boolean".into(), false),
                ("char".into(), false),
                ("name".into(), true)
            ]
        );
    }

    #[test]
    fn a_test_without_a_line_fails() {
        let scheduled = vec!["a".to_string(), "b".into()];
        assert_eq!(
            pass_set(&scheduled, &[("a".into(), true)]),
            [("a".to_string(), true), ("b".into(), false)]
        );
    }

    #[test]
    fn utc_times() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(1_791_391_000), "2026-10-07T16:36:40Z");
        assert_eq!(utc(951_782_400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn the_result_file_is_toml() {
        let run = Run {
            suite: "regress".into(),
            tests: vec![("a".into(), true), ("b\"".into(), false)],
            unstable: vec!["c".into()],
            seconds: 3,
        };
        let pins = crate::pins::Pins::builtin();
        let o = Oracle::new(&pins, 19, Path::new("/w")).unwrap();
        let server = Server { addr: "127.0.0.1:54319".parse().unwrap(), oracle: true };
        let t = crate::toml::parse(&run.to_toml(&o, &server, "2026-10-07T00:00:00Z")).unwrap();
        assert_eq!(t["passed"].as_int(), Some(1));
        assert_eq!(t["side"].as_str(), Some("oracle"));
        let one = |s: &str| crate::toml::Value::Array(vec![crate::toml::Value::Str(s.into())]);
        assert_eq!(t["fail"], one("b\""));
        assert_eq!(t["unstable"], one("c"));
    }
}
