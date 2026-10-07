//! The oracles: real PostgreSQL servers built from the pins of `pins.toml`.
//!
//! `rupg-compat oracle build` fetches the pinned commit with a shallow git fetch, runs configure with the options of `oracle/configure.txt`, builds and installs the server and the test programs under the work directory, and runs `initdb` with the fixed configuration of `oracle/`. See spec/21 section 21.3.3.
//!
//! The layout of the work directory for version N is `oracle/N/src` (the source and the build), `oracle/N/install`, `oracle/N/data`, `oracle/N/build.log` and `oracle/N/server.log`.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::pins::{Pin, Pins};

/// The configure options of every oracle.
const CONFIGURE: &str = include_str!("../oracle/configure.txt");
/// The lines that the build adds to `postgresql.conf`.
const POSTGRESQL_CONF: &str = include_str!("../oracle/postgresql.conf");
/// The host rules of every oracle.
const PG_HBA_CONF: &str = include_str!("../oracle/pg_hba.conf");
/// The fixed test certificate and its key.
const SERVER_CRT: &str = include_str!("../oracle/server.crt");
const SERVER_KEY: &str = include_str!("../oracle/server.key");

/// The superuser of every oracle.
pub(crate) const USER: &str = "postgres";
/// The SCRAM password of the superuser. It is a test password for a server that listens on 127.0.0.1 only.
pub(crate) const PASSWORD: &str = "rupg-compat";

/// The port of the oracle of a major version: 54314 for 14 to 54319 for 19.
pub(crate) fn port(major: u32) -> u16 {
    54300 + major as u16
}

/// The configure options, without comments and blank lines.
pub(crate) fn configure_options() -> Vec<&'static str> {
    CONFIGURE.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

/// The paths of one oracle in the work directory.
#[derive(Clone, Debug)]
pub(crate) struct Oracle {
    pub(crate) pin: Pin,
    pub(crate) dir: PathBuf,
}

impl Oracle {
    pub(crate) fn new(pins: &Pins, major: u32, work: &Path) -> Result<Oracle, String> {
        let pin = pins
            .postgres
            .iter()
            .find(|p| p.major == major)
            .ok_or_else(|| format!("no oracle pin for version {major} in pins.toml"))?;
        Ok(Oracle { pin: pin.clone(), dir: work.join("oracle").join(major.to_string()) })
    }

    pub(crate) fn src(&self) -> PathBuf {
        self.dir.join("src")
    }

    pub(crate) fn install(&self) -> PathBuf {
        self.dir.join("install")
    }

    pub(crate) fn bin(&self, program: &str) -> PathBuf {
        self.install().join("bin").join(program)
    }

    pub(crate) fn data(&self) -> PathBuf {
        self.dir.join("data")
    }

    pub(crate) fn port(&self) -> u16 {
        port(self.pin.major)
    }

    fn stamp(&self, step: &str) -> PathBuf {
        self.dir.join("stamps").join(step)
    }

    /// What a step depends on. A step runs again when this text changes.
    fn stamp_text(&self, step: &str) -> String {
        match step {
            "fetch" => self.pin.commit.clone(),
            "build" => {
                format!("{}\n{CONFIGURE}\n{}", self.pin.commit, extra_configure().join("\n"))
            }
            _ => format!("{}\n{POSTGRESQL_CONF}\n{PG_HBA_CONF}\n{SERVER_CRT}", self.pin.commit),
        }
    }

    fn done(&self, step: &str) -> bool {
        fs::read_to_string(self.stamp(step)).is_ok_and(|s| s == self.stamp_text(step))
    }

    fn mark(&self, step: &str) -> Result<(), String> {
        let path = self.stamp(step);
        fs::create_dir_all(path.parent().unwrap_or(&self.dir)).map_err(|e| e.to_string())?;
        fs::write(&path, self.stamp_text(step)).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Builds the oracle and runs `initdb`. A step that is done for the same pin and the same configuration is skipped, unless `force` is set.
    pub(crate) fn build(&self, repository: &str, jobs: u32, force: bool) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let log = self.dir.join("build.log");
        let mut steps = Steps::new(&log)?;
        let total = Instant::now();
        println!("oracle {}: {} at {}", self.pin.major, self.pin.git_ref, self.pin.commit);
        if force || !self.done("fetch") {
            self.fetch(&mut steps, repository)?;
            self.mark("fetch")?;
        }
        if force || !self.done("build") {
            self.compile(&mut steps, jobs)?;
            self.mark("build")?;
        }
        if force || !self.done("initdb") {
            self.initdb(&mut steps)?;
            self.mark("initdb")?;
        }
        println!(
            "oracle {}: ready in {:.1} s, log in {}",
            self.pin.major,
            total.elapsed().as_secs_f64(),
            log.display()
        );
        Ok(())
    }

    fn fetch(&self, steps: &mut Steps, repository: &str) -> Result<(), String> {
        let src = self.src();
        if !src.join(".git").exists() {
            steps.run("git init", Command::new("git").args(["init", "-q"]).arg(&src))?;
        }
        steps.run(
            "git fetch",
            Command::new("git").arg("-C").arg(&src).args([
                "fetch",
                "-q",
                "--depth",
                "1",
                repository,
                &self.pin.commit,
            ]),
        )?;
        steps.run(
            "git checkout",
            Command::new("git").arg("-C").arg(&src).args([
                "checkout",
                "-q",
                "-f",
                "--detach",
                "FETCH_HEAD",
            ]),
        )?;
        let head = output(Command::new("git").arg("-C").arg(&src).args(["rev-parse", "HEAD"]))?;
        if head.trim() != self.pin.commit {
            return Err(format!(
                "the checkout is at {}, not at the pin {}",
                head.trim(),
                self.pin.commit
            ));
        }
        Ok(())
    }

    fn compile(&self, steps: &mut Steps, jobs: u32) -> Result<(), String> {
        let src = self.src();
        let mut configure = Command::new("./configure");
        configure.current_dir(&src).arg(format!("--prefix={}", self.install().display()));
        configure.args(configure_options()).args(extra_configure());
        // configure finds ICU with pkg-config. Without pkg-config, it takes the flags from the environment.
        if Command::new("pkg-config").arg("--version").output().is_err() {
            configure.env("ICU_CFLAGS", " ").env("ICU_LIBS", "-licui18n -licuuc -licudata");
        }
        steps.run("configure", &mut configure)?;
        let j = format!("-j{jobs}");
        steps.run("make", Command::new("make").current_dir(&src).args(["-s", &j]))?;
        steps
            .run("make install", Command::new("make").current_dir(&src).args(["-s", "install"]))?;
        // The test programs of the suites: pg_regress and regress.so, isolationtester, and libpq_pipeline.
        for dir in ["src/test/regress", "src/test/isolation", "src/test/modules/libpq_pipeline"] {
            steps.run(
                &format!("make {dir}"),
                Command::new("make").current_dir(src.join(dir)).args(["-s", &j]),
            )?;
        }
        Ok(())
    }

    fn initdb(&self, steps: &mut Steps) -> Result<(), String> {
        let data = self.data();
        if self.running() {
            self.stop()?;
        }
        if data.exists() {
            fs::remove_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
        }
        let pwfile = self.dir.join("pwfile");
        fs::write(&pwfile, format!("{PASSWORD}\n")).map_err(|e| e.to_string())?;
        let result = steps.run(
            "initdb",
            Command::new(self.bin("initdb"))
                .arg("-D")
                .arg(&data)
                .args([
                    "--no-locale",
                    "-E",
                    "UTF8",
                    "-U",
                    USER,
                    "--auth-local=trust",
                    "--auth-host=scram-sha-256",
                ])
                .arg(format!("--pwfile={}", pwfile.display()))
                .env("TZ", "UTC"),
        );
        let _ = fs::remove_file(&pwfile);
        result?;
        let conf = data.join("postgresql.conf");
        let mut f = fs::OpenOptions::new().append(true).open(&conf).map_err(|e| e.to_string())?;
        write!(
            f,
            "\n{POSTGRESQL_CONF}\n# Written by rupg-compat for the oracle of {}.\nport = {}\n",
            self.pin.major,
            self.port()
        )
        .map_err(|e| e.to_string())?;
        fs::write(data.join("pg_hba.conf"), PG_HBA_CONF).map_err(|e| e.to_string())?;
        fs::write(data.join("server.crt"), SERVER_CRT).map_err(|e| e.to_string())?;
        fs::write(data.join("server.key"), SERVER_KEY).map_err(|e| e.to_string())?;
        set_owner_only(&data.join("server.key"))?;
        Ok(())
    }

    /// True when the postmaster of this oracle runs.
    pub(crate) fn running(&self) -> bool {
        Command::new(self.bin("pg_ctl"))
            .arg("-D")
            .arg(self.data())
            .arg("status")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    pub(crate) fn start(&self) -> Result<(), String> {
        if self.running() {
            return Ok(());
        }
        let status = Command::new(self.bin("pg_ctl"))
            .arg("-D")
            .arg(self.data())
            .arg("-l")
            .arg(self.dir.join("server.log"))
            .args(["-w", "-t", "120", "start"])
            .stdout(Stdio::null())
            .status()
            .map_err(|e| format!("pg_ctl: {e}"))?;
        if !status.success() {
            return Err(format!(
                "the oracle of {} did not start, see {}",
                self.pin.major,
                self.dir.join("server.log").display()
            ));
        }
        Ok(())
    }

    pub(crate) fn stop(&self) -> Result<(), String> {
        let status = Command::new(self.bin("pg_ctl"))
            .arg("-D")
            .arg(self.data())
            .args(["-m", "fast", "-w", "stop"])
            .stdout(Stdio::null())
            .status()
            .map_err(|e| format!("pg_ctl: {e}"))?;
        if !status.success() {
            return Err(format!("the oracle of {} did not stop", self.pin.major));
        }
        Ok(())
    }

    /// Runs one statement with the psql of the oracle, over TCP with TLS and the SCRAM password.
    pub(crate) fn psql(&self, sql: &str) -> Result<String, String> {
        let out = output(
            Command::new(self.bin("psql"))
                .args([
                    "-X",
                    "-A",
                    "-t",
                    "-h",
                    "127.0.0.1",
                    "-U",
                    USER,
                    "-d",
                    "postgres",
                    "-c",
                    sql,
                ])
                .arg("-p")
                .arg(self.port().to_string())
                .env("PGPASSWORD", PASSWORD)
                .env("PGSSLMODE", "require"),
        )?;
        Ok(out.trim_end().to_string())
    }

    /// The environment that points libpq at this oracle over TCP.
    pub(crate) fn env(&self) -> Vec<(&'static str, String)> {
        vec![
            ("PGHOST", "127.0.0.1".into()),
            ("PGPORT", self.port().to_string()),
            ("PGUSER", USER.into()),
            ("PGPASSWORD", PASSWORD.into()),
            ("PGDATABASE", "postgres".into()),
        ]
    }
}

/// `--with-includes` and `--with-libraries` from `RUPG_COMPAT_INCLUDES` and `RUPG_COMPAT_LIBRARIES`, for a machine where a header is not in a standard place. They do not change the server.
fn extra_configure() -> Vec<String> {
    let mut out = Vec::new();
    for (var, option) in
        [("RUPG_COMPAT_INCLUDES", "--with-includes"), ("RUPG_COMPAT_LIBRARIES", "--with-libraries")]
    {
        if let Ok(v) = std::env::var(var)
            && !v.is_empty()
        {
            out.push(format!("{option}={v}"));
        }
    }
    out
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn set_owner_only(_: &Path) -> Result<(), String> {
    Ok(())
}

fn output(cmd: &mut Command) -> Result<String, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{cmd:?}: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Runs the steps of a build. Each step prints its command and its time, and writes its output to the log.
struct Steps {
    log: fs::File,
    path: PathBuf,
}

impl Steps {
    fn new(path: &Path) -> Result<Steps, String> {
        let log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Steps { log, path: path.to_path_buf() })
    }

    fn run(&mut self, name: &str, cmd: &mut Command) -> Result<(), String> {
        let line = format!("{cmd:?}");
        println!("  $ {line}");
        writeln!(self.log, "\n$ {line}").map_err(|e| e.to_string())?;
        let out = self.log.try_clone().map_err(|e| e.to_string())?;
        let err = self.log.try_clone().map_err(|e| e.to_string())?;
        let start = Instant::now();
        let status = cmd.stdout(out).stderr(err).status().map_err(|e| format!("{name}: {e}"))?;
        let secs = start.elapsed().as_secs_f64();
        println!("    {name}: {secs:.1} s");
        writeln!(self.log, "# {name}: {secs:.1} s, {status}").map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("{name} failed ({status}), see {}", self.path.display()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_do_not_overlap() {
        let pins = Pins::builtin();
        let mut ports: Vec<u16> = pins.postgres.iter().map(|p| port(p.major)).collect();
        ports.dedup();
        assert_eq!(ports.len(), 6);
        assert_eq!(port(19), 54319);
    }

    #[test]
    fn the_fixed_configuration_has_the_settings_of_the_spec() {
        for line in [
            "TimeZone = 'UTC'",
            "DateStyle = 'ISO, MDY'",
            "lc_messages = 'C'",
            "password_encryption = 'scram-sha-256'",
            "ssl = on",
        ] {
            assert!(POSTGRESQL_CONF.lines().any(|l| l == line), "{line}");
        }
        assert!(configure_options().contains(&"--with-ssl=openssl"));
        assert!(PG_HBA_CONF.contains("scram-sha-256"));
    }

    #[test]
    fn the_certificate_and_the_key_are_pem() {
        assert!(SERVER_CRT.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(SERVER_KEY.starts_with("-----BEGIN PRIVATE KEY-----"));
    }

    #[test]
    fn every_pin_has_an_oracle_dir() {
        let pins = Pins::builtin();
        let o = Oracle::new(&pins, 18, Path::new("/w")).unwrap();
        assert_eq!(o.data(), Path::new("/w/oracle/18/data"));
        assert!(Oracle::new(&pins, 13, Path::new("/w")).is_err());
    }
}
