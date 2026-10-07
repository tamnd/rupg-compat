//! The PostgreSQL compatibility harness for rupg.
//!
//! The runner sends one input to a PostgreSQL oracle and to rupg and compares the two answers. The levels L1 to L5 and the suites are in `spec/05-compatibility.md` and `spec/21-testing.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Prints a line to stdout like `println!`, but a closed pipe, as in `rupg-compat replay T | head`, is not a panic.
macro_rules! out {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stdout(), $($arg)*);
    }};
}

mod args;
mod client;
mod compare;
mod diff;
mod frame;
mod import;
mod levels;
mod message;
mod oracle;
mod pins;
mod proxy;
mod replay;
mod scram;
mod suites;
mod toml;
mod trace;

use args::Args;
use oracle::Oracle;
use pins::Pins;

const USAGE: &str = "usage: rupg-compat <command> [options]

commands:
  pins                      print the pins of pins.toml
  levels                    print the five levels
  oracle build              fetch, build and initdb the oracle (--all for the six, --jobs N, --force)
  oracle start | stop       start or stop the oracle (--all for the six)
  oracle status             print the pin, the port and version() of each built oracle
  oracle env                print the libpq environment of the oracle
  proxy --out FILE          record every connection to the oracle until stopped (--listen ADDR)
  record <name> [-- CMD]    run CMD, or clients/<name>/run.sh, through the proxy and write a trace
                            (default: corpus/traces/<name>.trace, --out FILE)
  replay TRACE...           reset the databases of each trace, send it to the oracle twice to find the unstable
                            groups, then to the server under test, and compare the answers
                            (--server ADDR for another server, --timeout S, --out FILE, --show N)
  diff SQL                  run one statement on the oracle and on the server under test, compare the answers
                            and show the first different byte (--server ADDR, --database DB)
  trace FILE...             check that each line of a trace parses and count its messages
  import                    write corpus/postgres/N/ from the source of the oracle at its pin: the SHA-256
                            manifest of the suites, the lists and the counts (--all, --check, --corpus DIR)
  regress                   run the 239 scheduled regression tests with pg_regress of the oracle build
  isolation                 run the isolation specs with pg_isolation_regress: the schedule, then the specs
                            that it does not list
  pipeline                  run each test of libpq_pipeline and compare the 9 libpq traces
                            (each suite: --server ADDR for the server under test, else the oracle; --corpus DIR;
                            it writes results/N/<suite>-<oracle|server>.toml in the work directory)

options:
  --compat-version N        the oracle of version N (default: the reference of pins.toml); with --server,
                            the startup message also sets rupg.compat_version to N on the server under test
  --version N               the same as --compat-version N, for the oracle commands only
  --work DIR                the work directory (default: $RUPG_COMPAT_WORK, then ./work)";

fn main() -> ExitCode {
    let args = match Args::parse(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("rupg-compat: {e}");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("rupg-compat: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let pins = Pins::builtin();
    match args.word(0) {
        Some("pins") => {
            for pin in &pins.postgres {
                out!("{:<4} {:<10} {:<16} {}", pin.major, pin.release, pin.git_ref, pin.commit);
            }
            out!("rupg {}", pins.rupg_commit);
            for client in &pins.clients {
                out!("client {} {}", client.name, client.version);
            }
        }
        Some("levels") => {
            for level in levels::LEVELS {
                out!("{}  {}", level.id, level.name);
            }
        }
        Some("oracle") => oracle_command(args, &pins)?,
        Some("proxy") => proxy_command(args, &pins)?,
        Some("record") => return record_command(args, &pins),
        Some("trace") => trace_command(args)?,
        Some("replay") => return replay_command(args, &pins),
        Some("diff") => return diff_command(args, &pins),
        Some("import") => return import_command(args, &pins),
        Some(suite @ ("regress" | "isolation" | "pipeline")) => {
            return suite_command(args, &pins, suite);
        }
        None if args.flag("help") => out!("{USAGE}"),
        _ if args.words.is_empty() && args.rest.is_empty() => {
            eprintln!("{USAGE}");
            return Ok(ExitCode::from(2));
        }
        Some(other) => return Err(format!("unknown command {other}\n{USAGE}")),
        None => return Err(USAGE.into()),
    }
    Ok(ExitCode::SUCCESS)
}

fn oracle_command(args: &Args, pins: &Pins) -> Result<(), String> {
    args.only(&["compat-version", "version", "all", "jobs", "force"])?;
    let work = args.work_dir();
    // Spec/21 section 21.3.4 writes `oracle build --version V`, and every command takes `--compat-version V`. Both work here.
    let version = match args.number("version")? {
        Some(_) if args.get("compat-version").is_some() => {
            return Err("give --version or --compat-version, not both".into());
        }
        Some(v) => v,
        None => args.compat_version(pins.reference)?,
    };
    let majors: Vec<u32> = if args.flag("all") {
        pins.postgres.iter().rev().map(|p| p.major).collect()
    } else {
        vec![version]
    };
    let oracles =
        majors.iter().map(|&m| Oracle::new(pins, m, &work)).collect::<Result<Vec<_>, _>>()?;
    match args.word(1) {
        Some("build") => {
            let jobs = args.number("jobs")?.unwrap_or(4);
            for o in &oracles {
                o.build(&pins.repository, jobs, args.flag("force"))?;
            }
        }
        Some("start") => {
            for o in &oracles {
                o.start()?;
                out!("oracle {} runs on 127.0.0.1:{}", o.pin.major, o.port());
            }
        }
        Some("stop") => {
            for o in oracles.iter().filter(|o| o.running()) {
                o.stop()?;
                out!("oracle {} stopped", o.pin.major);
            }
        }
        Some("status") => {
            for o in &oracles {
                let state = if !o.bin("postgres").exists() {
                    "not built".to_string()
                } else if o.running() {
                    o.psql("SELECT version()").unwrap_or_else(|e| e)
                } else {
                    "stopped".to_string()
                };
                out!("{:<4} {:<10} port {}  {state}", o.pin.major, o.pin.release, o.port());
            }
        }
        Some("env") => {
            for (k, v) in oracles[0].env() {
                out!("export {k}={v}");
            }
        }
        _ => {
            return Err(format!(
                "usage: rupg-compat oracle <build | start | stop | status | env>\n{USAGE}"
            ));
        }
    }
    Ok(())
}

/// The oracle that the proxy forwards to. It must run.
fn running_oracle(args: &Args, pins: &Pins) -> Result<Oracle, String> {
    let o = Oracle::new(pins, args.compat_version(pins.reference)?, &args.work_dir())?;
    if !o.running() {
        return Err(format!(
            "the oracle of {} does not run, start it with: rupg-compat oracle start --compat-version {}",
            o.pin.major, o.pin.major
        ));
    }
    Ok(o)
}

fn trace_header(o: &Oracle, what: &str) -> Vec<String> {
    vec![
        "rupg-compat trace".into(),
        format!("compat-version: {}", o.pin.major),
        format!("server: PostgreSQL {} {}", o.pin.release, o.pin.commit),
        format!("recorded: {what}"),
    ]
}

fn oracle_addr(o: &Oracle) -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], o.port()))
}

/// The database that `record` gives to a client.
const RECORD_DATABASE: &str = "compat";

fn oracle_target(o: &Oracle) -> client::Target {
    client::Target {
        addr: oracle_addr(o),
        user: oracle::USER.into(),
        password: oracle::PASSWORD.into(),
        params: Vec::new(),
    }
}

/// The server under test: the oracle itself, or `--server ADDR` with `rupg.compat_version` set to the version of the oracle (spec/21 section 21.3.4).
fn server_target(args: &Args, o: &Oracle) -> Result<client::Target, String> {
    let mut target = oracle_target(o);
    if let Some(server) = args.get("server") {
        target.addr = server.parse().map_err(|e| format!("--server {server}: {e}"))?;
        target.params.push(("rupg.compat_version".into(), o.pin.major.to_string()));
    }
    Ok(target)
}

fn proxy_command(args: &Args, pins: &Pins) -> Result<(), String> {
    args.only(&["compat-version", "listen", "out"])?;
    let o = running_oracle(args, pins)?;
    let out = PathBuf::from(args.get("out").ok_or("proxy needs --out FILE")?);
    let listen = args.get("listen").unwrap_or("127.0.0.1:0");
    let p = proxy::Proxy::start(listen, oracle_addr(&o), &out, &trace_header(&o, "proxy"))?;
    out!(
        "proxy on {} for the oracle of {} on port {}, trace in {}",
        p.addr,
        o.pin.major,
        o.port(),
        out.display()
    );
    out!("stop it with Control-C; the proxy writes each line when it sees the message");
    p.finish()
}

fn record_command(args: &Args, pins: &Pins) -> Result<ExitCode, String> {
    args.only(&["compat-version", "out"])?;
    let name = args.word(1).ok_or("usage: rupg-compat record <name> [-- command]")?;
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)) {
        return Err(format!("the trace name {name:?} must be letters, digits, '-', '_' and '.'"));
    }
    let o = running_oracle(args, pins)?;
    let mut cmd = match args.rest.split_first() {
        Some((program, rest)) => {
            let mut c = Command::new(program);
            c.args(rest);
            c
        }
        None => {
            let script = Path::new("clients").join(name).join("run.sh");
            if !script.exists() {
                return Err(format!(
                    "{} does not exist, give a command after --",
                    script.display()
                ));
            }
            let mut c = Command::new("sh");
            c.arg(script);
            c
        }
    };
    let out = match args.get("out") {
        Some(f) => PathBuf::from(f),
        None => Path::new("corpus").join("traces").join(format!("{name}.trace")),
    };
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let what =
        if args.rest.is_empty() { format!("clients/{name}/run.sh") } else { args.rest.join(" ") };
    // Each recording starts from a new database, and replay makes the same database again.
    client::reset_database(&oracle_target(&o), RECORD_DATABASE)?;
    let p = proxy::Proxy::start("127.0.0.1:0", oracle_addr(&o), &out, &trace_header(&o, &what))?;
    // The proxy does not speak TLS, so the client must not require it.
    for (k, v) in o.env() {
        cmd.env(k, v);
    }
    cmd.env("PGPORT", p.addr.port().to_string())
        .env("PGSSLMODE", "disable")
        .env("PGDATABASE", RECORD_DATABASE);
    let status = cmd.status().map_err(|e| format!("{what}: {e}"));
    p.finish()?;
    let status = status?;
    let text = std::fs::read_to_string(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let lines = text.lines().filter(|l| !l.starts_with('#')).count();
    out!("{}: {lines} lines, the command exited with {status}", out.display());
    Ok(if status.success() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

fn trace_command(args: &Args) -> Result<(), String> {
    args.only(&[])?;
    if args.words.len() < 2 {
        return Err("usage: rupg-compat trace FILE...".into());
    }
    for file in &args.words[1..] {
        let t = trace::Trace::read(Path::new(file))?;
        let counts = trace::check(&t).map_err(|e| format!("{file}: {e}"))?;
        let conns: std::collections::BTreeSet<u32> = t.lines.iter().map(|l| l.conn).collect();
        let msgs: usize = counts.values().sum();
        let version = t.header_value("compat-version").unwrap_or("not given");
        let mut text = format!(
            "{file}: compat-version {version}, {} connections, {msgs} messages\n",
            conns.len()
        );
        for (name, n) in counts {
            text.push_str(&format!("  {n:>8}  {name}\n"));
        }
        // A reader such as `head` can close the pipe early. That is not an error of the trace.
        if std::io::Write::write_all(&mut std::io::stdout(), text.as_bytes()).is_err() {
            return Ok(());
        }
    }
    Ok(())
}

/// The databases that the connections of a trace open.
fn trace_databases(t: &trace::Trace) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for m in t.lines.iter().filter_map(trace::Line::msg).filter(|m| m.name() == "StartupMessage") {
        let pairs = m.vals.get(1).map(message::Val::list).unwrap_or_default();
        let get = |key: &[u8]| {
            pairs
                .chunks(2)
                .find(|p| p[0].bytes() == Some(key))
                .and_then(|p| p.get(1)?.bytes())
                .map(|b| String::from_utf8_lossy(b).into_owned())
        };
        if let Some(db) = get(b"database").or_else(|| get(b"user"))
            && !out.contains(&db)
        {
            out.push(db);
        }
    }
    out
}

fn replay_command(args: &Args, pins: &Pins) -> Result<ExitCode, String> {
    args.only(&["compat-version", "server", "timeout", "out", "show"])?;
    let files = &args.words[1..];
    if files.is_empty() {
        return Err("usage: rupg-compat replay TRACE... [--server ADDR] [--out FILE]".into());
    }
    if args.get("out").is_some() && files.len() > 1 {
        return Err("--out takes one trace".into());
    }
    let o = running_oracle(args, pins)?;
    let oracle = oracle_target(&o);
    let target = server_target(args, &o)?;
    let timeout = std::time::Duration::from_secs(args.number("timeout")?.unwrap_or(30));
    let mut failed = false;
    let mut total = compare::Outcome::default();
    for file in files {
        let t = trace::Trace::read(Path::new(file))?;
        let reset = |server: &client::Target| {
            for db in trace_databases(&t) {
                if let Err(e) = client::reset_database(server, &db) {
                    out!("{file}: {e}");
                }
            }
        };
        reset(&oracle);
        // The unstable filter (spec/21 section 21.1): two runs on the oracle, and the groups that differ between them leave the comparison.
        let first = replay::replay(&t, &oracle, timeout);
        reset(&oracle);
        let second = replay::replay(&t, &oracle, timeout);
        let unstable = compare::unstable(&first.lines, &second.lines);
        // A note of the second run, such as a timeout, can make a group look unstable, so it fails the run too.
        let notes: Vec<String> =
            second.notes.iter().map(|n| format!("second oracle run: {n}")).collect();
        let r = if args.get("server").is_none() {
            first
        } else {
            reset(&target);
            replay::replay(&t, &target, timeout)
        };
        let mut o = compare::compare(&t.lines, &r.lines, &unstable);
        o.items[1].applied += r.oids.applied;
        o.items[1].differed += r.oids.applied;
        let (pairs, gaps) = r.oids.gaps();
        out!(
            "{file}: {} groups, {} backend messages, {} unstable groups, {} groups differ; {} OIDs mapped, {gaps} of {pairs} OID gaps differ, {} SCRAM exchanges, {} cancel keys rewritten",
            o.groups,
            o.messages,
            o.unstable,
            o.diffs.len(),
            r.oids.len(),
            r.scram,
            r.cancels
        );
        for ((conn, group), d) in o.diffs.iter().take(args.number("show")?.unwrap_or(5)) {
            out!("  connection {conn} group {group}: {d}");
        }
        for n in r.notes.iter().chain(&notes) {
            out!("  {n}");
        }
        total.add(&o);
        failed |= !o.diffs.is_empty() || !r.notes.is_empty() || !notes.is_empty() || gaps > 0;
        if let Some(out) = args.get("out") {
            let replayed = trace::Trace { header: t.header.clone(), lines: r.lines };
            std::fs::write(out, replayed.to_text()).map_err(|e| format!("{out}: {e}"))?;
        }
    }
    print_exclusions(&total);
    Ok(if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

fn diff_command(args: &Args, pins: &Pins) -> Result<ExitCode, String> {
    args.only(&["compat-version", "server", "database"])?;
    let sql = args.words[1..].join(" ");
    if sql.trim().is_empty() {
        return Err("usage: rupg-compat diff SQL [--server ADDR] [--database DB]".into());
    }
    let o = running_oracle(args, pins)?;
    let oracle = oracle_target(&o);
    let server = server_target(args, &o)?;
    let database = args.get("database").unwrap_or("postgres");
    let want = client::Conn::connect(&oracle, database)?.query(&sql)?;
    let got = client::Conn::connect(&server, database)?.query(&sql)?;
    let lines = |msgs: &[message::Msg]| {
        let query = message::Msg::new(message::Dir::F, "Query", vec![client::str_val(&sql)]);
        std::iter::once((message::Dir::F, query))
            .chain(msgs.iter().map(|m| (message::Dir::B, m.clone())))
            .map(|(dir, m)| trace::Line { conn: 1, dir, event: trace::Event::Msg(m) })
            .collect::<Vec<_>>()
    };
    let outcome = compare::compare(&lines(&want), &lines(&got), &Default::default());
    out!("oracle {} at {}: {} messages", o.pin.major, oracle.addr, want.len());
    out!("server at {}: {} messages", server.addr, got.len());
    match diff::first_difference(&want, &got) {
        None => out!("the answers are the same, byte for byte"),
        Some(d) => {
            let byte = |b: Option<u8>| b.map_or("no byte".to_string(), |b| format!("0x{b:02x}"));
            out!(
                "the first different byte is at offset {}, byte {} of message {}: {} on the oracle, {} on the server",
                d.offset,
                d.within,
                d.message + 1,
                byte(d.want),
                byte(d.got)
            );
            for (side, msgs) in [("oracle", &want), ("server", &got)] {
                let shown = msgs.get(d.message).map_or("no message".into(), trace::format_msg);
                out!("  {side}: {shown}");
            }
        }
    }
    if outcome.diffs.is_empty() {
        out!("after the rules and the exclusions, the answers are equal");
    } else {
        for d in outcome.diffs.values() {
            out!("after the rules and the exclusions: {d}");
        }
    }
    print_exclusions(&outcome);
    Ok(if outcome.diffs.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

fn import_command(args: &Args, pins: &Pins) -> Result<ExitCode, String> {
    args.only(&["compat-version", "all", "check", "corpus"])?;
    let corpus = PathBuf::from(args.get("corpus").unwrap_or("corpus"));
    let majors: Vec<u32> = if args.flag("all") {
        pins.postgres.iter().rev().map(|p| p.major).collect()
    } else {
        vec![args.compat_version(pins.reference)?]
    };
    let mut failed = false;
    for major in majors {
        let o = Oracle::new(pins, major, &args.work_dir())?;
        let src = o.src();
        let head = Command::new("git")
            .arg("-C")
            .arg(&src)
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| format!("git: {e}"))?;
        let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
        if head != o.pin.commit {
            return Err(format!(
                "{} is at {head:?}, not at the pin {}; run rupg-compat oracle build --compat-version {major}",
                src.display(),
                o.pin.commit
            ));
        }
        let imp = import::import(&src, major, &o.pin.commit)?;
        let dir = import::corpus_dir(&corpus, major);
        if args.flag("check") {
            let differ = import::check(&dir, &imp);
            if differ.is_empty() {
                out!("{}: equal to the source at {}", dir.display(), o.pin.commit);
            } else {
                out!(
                    "{}: these files differ from the source at {}: {}",
                    dir.display(),
                    o.pin.commit,
                    differ.join(", ")
                );
                failed = true;
            }
        } else {
            std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            for (name, text) in &imp.outputs {
                std::fs::write(dir.join(name), text).map_err(|e| format!("{name}: {e}"))?;
            }
            let files = imp.outputs[0].1.lines().count();
            out!("{}: {files} files in the manifest at {}", dir.display(), o.pin.commit);
        }
        for c in &imp.counts {
            out!("  {}.{} = {}", c.table, c.key, c.value);
        }
    }
    Ok(if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

/// Runs one PostgreSQL suite against the oracle or, with `--server`, against the server under test.
fn suite_command(args: &Args, pins: &Pins, suite: &str) -> Result<ExitCode, String> {
    args.only(&["compat-version", "server", "corpus"])?;
    let work = args.work_dir();
    let corpus = PathBuf::from(args.get("corpus").unwrap_or("corpus"));
    let server = match args.get("server") {
        Some(s) => suites::Server {
            addr: s.parse().map_err(|e| format!("--server {s}: {e}"))?,
            oracle: false,
        },
        None => suites::Server { addr: oracle_addr(&running_oracle(args, pins)?), oracle: true },
    };
    let o = Oracle::new(pins, args.compat_version(pins.reference)?, &work)?;
    let started = suites::utc_now();
    let run = match suite {
        "regress" => suites::regress(&o, &corpus, &server, &work)?,
        "isolation" => suites::isolation(&o, &corpus, &server, &work)?,
        _ => suites::pipeline(&o, &corpus, &server, &work)?,
    };
    suites::report_run(&run, &o, &server, &work, &started)?;
    Ok(if run.failed().is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

/// Prints each exclusion of spec/05 section 5.3 and each rule of section 5.2 with its counts.
fn print_exclusions(o: &compare::Outcome) {
    out!("exclusions (spec/05 section 5.3): applied, differed");
    for (i, (name, c)) in compare::ITEMS.iter().zip(o.items).enumerate() {
        out!("  {:>2}  {:>8} {:>8}  {name}", i + 1, c.applied, c.differed);
    }
    out!("rules (spec/05 section 5.2): applied, differed");
    for (name, c) in compare::RULES.iter().zip(o.rules) {
        out!("      {:>8} {:>8}  {name}", c.applied, c.differed);
    }
}
