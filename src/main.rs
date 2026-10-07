//! The PostgreSQL compatibility harness for rupg.
//!
//! The runner sends one input to a PostgreSQL oracle and to rupg and compares the two answers. The levels L1 to L5 and the suites are in `spec/05-compatibility.md` and `spec/21-testing.md` of tamnd/rupg.

#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

mod args;
mod frame;
mod levels;
mod message;
mod oracle;
mod pins;
mod proxy;
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
  trace FILE...             check that each line of a trace parses and count its messages

options:
  --compat-version N        the oracle of version N (default: the reference of pins.toml)
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
                println!("{:<4} {:<10} {:<16} {}", pin.major, pin.release, pin.git_ref, pin.commit);
            }
            println!("rupg {}", pins.rupg_commit);
            for client in &pins.clients {
                println!("client {} {}", client.name, client.version);
            }
        }
        Some("levels") => {
            for level in levels::LEVELS {
                println!("{}  {}", level.id, level.name);
            }
        }
        Some("oracle") => oracle_command(args, &pins)?,
        Some("proxy") => proxy_command(args, &pins)?,
        Some("record") => return record_command(args, &pins),
        Some("trace") => trace_command(args)?,
        None if args.flag("help") => println!("{USAGE}"),
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
    args.only(&["compat-version", "all", "jobs", "force"])?;
    let work = args.work_dir();
    let majors: Vec<u32> = if args.flag("all") {
        pins.postgres.iter().rev().map(|p| p.major).collect()
    } else {
        vec![args.compat_version(pins.reference)?]
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
                println!("oracle {} runs on 127.0.0.1:{}", o.pin.major, o.port());
            }
        }
        Some("stop") => {
            for o in oracles.iter().filter(|o| o.running()) {
                o.stop()?;
                println!("oracle {} stopped", o.pin.major);
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
                println!("{:<4} {:<10} port {}  {state}", o.pin.major, o.pin.release, o.port());
            }
        }
        Some("env") => {
            for (k, v) in oracles[0].env() {
                println!("export {k}={v}");
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

fn proxy_command(args: &Args, pins: &Pins) -> Result<(), String> {
    args.only(&["compat-version", "listen", "out"])?;
    let o = running_oracle(args, pins)?;
    let out = PathBuf::from(args.get("out").ok_or("proxy needs --out FILE")?);
    let listen = args.get("listen").unwrap_or("127.0.0.1:0");
    let p = proxy::Proxy::start(listen, oracle_addr(&o), &out, &trace_header(&o, "proxy"))?;
    println!(
        "proxy on {} for the oracle of {} on port {}, trace in {}",
        p.addr,
        o.pin.major,
        o.port(),
        out.display()
    );
    println!("stop it with Control-C; the proxy writes each line when it sees the message");
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
    let p = proxy::Proxy::start("127.0.0.1:0", oracle_addr(&o), &out, &trace_header(&o, &what))?;
    // The proxy does not speak TLS, so the client must not require it.
    for (k, v) in o.env() {
        cmd.env(k, v);
    }
    cmd.env("PGPORT", p.addr.port().to_string()).env("PGSSLMODE", "disable");
    let status = cmd.status().map_err(|e| format!("{what}: {e}"));
    p.finish()?;
    let status = status?;
    let text = std::fs::read_to_string(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let lines = text.lines().filter(|l| !l.starts_with('#')).count();
    println!("{}: {lines} lines, the command exited with {status}", out.display());
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
