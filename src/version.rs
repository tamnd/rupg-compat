//! The version check: the four places where rupg reports its version have the forms of spec/05 section 5.7.
//!
//! A client reads the version from `server_version` at startup, from `SHOW server_version_num` and from `version()`. Each must start with the version of PostgreSQL, because clients parse it. `rupg.version` gives the version of rupg, and the server sends it after the standard parameters. The check runs only against the server under test, because the oracle does not have these forms.

use std::time::Instant;

use crate::client::Conn;
use crate::introspect::values;
use crate::oracle::Oracle;
use crate::suites::{Run, Server};

/// The parameters that some clients use to find another system. rupg must not send them.
const OTHER_SYSTEMS: [&str; 3] = ["crdb_version", "mz_version", "questdb_version"];

/// What the server gave in the four places.
#[derive(Debug, Default)]
pub(crate) struct Seen {
    /// The `ParameterStatus` messages of the startup, in their order.
    pub(crate) status: Vec<(String, String)>,
    pub(crate) show_version: String,
    pub(crate) version_num: String,
    pub(crate) version: String,
    pub(crate) show_rupg: String,
    /// The error of `SET rupg.version`, which must be the error of a parameter with the context `internal`.
    pub(crate) set_rupg: String,
}

/// One check, with what the server gave when it fails.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Check {
    pub(crate) name: &'static str,
    pub(crate) ok: bool,
    pub(crate) got: String,
}

/// The number of standard parameters that version `major` reports at startup (spec/05 section 5.6.2).
fn reported(major: u32) -> usize {
    match major {
        14 | 15 => 13,
        16 | 17 => 14,
        _ => 15,
    }
}

/// True when `s` is one or more ASCII digits.
fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// The major version, the minor version and the rupg version of `N.M (rupg X.Y.Z)`.
fn parse_server_version(s: &str) -> Option<(u32, u32, &str)> {
    let (number, rest) = s.split_once(' ')?;
    let (major, minor) = number.split_once('.')?;
    let rupg = rest.strip_prefix("(rupg ")?.strip_suffix(')')?;
    let parts: Vec<&str> = rupg.split('.').collect();
    if !digits(major) || !digits(minor) || parts.len() != 3 || !parts.iter().all(|p| digits(p)) {
        return None;
    }
    Some((major.parse().ok()?, minor.parse().ok()?, rupg))
}

/// True when `s` is `PostgreSQL <number> on <target>, compiled by rustc <version>, <32 or 64>-bit`.
fn version_form(s: &str, number: &str) -> bool {
    let Some(rest) = s.strip_prefix(&format!("PostgreSQL {number} on ")) else { return false };
    let Some((target, rest)) = rest.split_once(", compiled by rustc ") else { return false };
    let Some((rustc, bits)) = rest.split_once(", ") else { return false };
    let target_ok = !target.is_empty() && !target.contains([' ', ',']);
    let rustc_ok = rustc.starts_with(|c: char| c.is_ascii_digit()) && !rustc.contains(',');
    target_ok && rustc_ok && (bits == "64-bit" || bits == "32-bit")
}

/// The checks of spec/05 section 5.7 for a server that runs with `rupg.compat_version` set to `major`.
pub(crate) fn checks(major: u32, seen: &Seen) -> Vec<Check> {
    let status = |name: &str| seen.status.iter().find(|p| p.0 == name).map(|p| p.1.as_str());
    let server_version = status("server_version").unwrap_or_default();
    let parsed = parse_server_version(server_version);
    let rupg = parsed.map(|p| p.2).unwrap_or_default();
    let mut out = Vec::new();
    let mut put = |name: &'static str, ok: bool, got: String| out.push(Check { name, ok, got });
    put(
        "server_version",
        parsed.is_some_and(|p| p.0 == major),
        format!("{server_version:?}, want \"{major}.M (rupg X.Y.Z)\""),
    );
    put(
        "SHOW server_version",
        seen.show_version == server_version,
        format!("{:?}, want the value of the startup {server_version:?}", seen.show_version),
    );
    let num = parsed.map(|p| (p.0 * 10_000 + p.1).to_string()).unwrap_or_default();
    put(
        "server_version_num",
        parsed.is_some() && seen.version_num == num,
        format!("{:?}, want {num:?}", seen.version_num),
    );
    let number = server_version.split(')').next().unwrap_or_default().to_string() + ")";
    put(
        "version()",
        parsed.is_some() && version_form(&seen.version, &number),
        format!(
            "{:?}, want \"PostgreSQL {number} on <target>, compiled by rustc <version>, 64-bit\"",
            seen.version
        ),
    );
    let reported_rupg = status("rupg.version").unwrap_or_default();
    put(
        "rupg.version",
        parsed.is_some() && reported_rupg == rupg && seen.show_rupg == rupg,
        format!(
            "startup {reported_rupg:?} and SHOW {:?}, want {rupg:?} of server_version",
            seen.show_rupg
        ),
    );
    let cannot = "ERROR 55P02 parameter \"rupg.version\" cannot be changed";
    put(
        "rupg.version is internal",
        seen.set_rupg == cannot,
        format!("{:?}, want {cannot:?}", seen.set_rupg),
    );
    let names: Vec<&str> = seen.status.iter().map(|p| p.0.as_str()).collect();
    let want = reported(major);
    put(
        "rupg.version after the standard parameters",
        names.len() == want + 1 && names.last() == Some(&"rupg.version"),
        format!("{} parameters {names:?}, want {want} and then rupg.version", names.len()),
    );
    let other: Vec<&str> = names.iter().copied().filter(|n| OTHER_SYSTEMS.contains(n)).collect();
    put("no parameter of another system", other.is_empty(), format!("{other:?}"));
    out
}

/// Connects to the server under test, reads the four places and checks them.
pub(crate) fn run(o: &Oracle, server: &Server) -> Result<(Run, Vec<Check>), String> {
    let started = Instant::now();
    let mut c = Conn::connect(&server.target(o), "postgres")?;
    // An error is a value that fails its check, so the other checks still run.
    let mut one = |sql: &str| -> Result<String, String> {
        let msgs = c.query(sql)?;
        if let Some(e) = msgs.iter().find(|m| m.name() == "ErrorResponse") {
            return Ok(crate::client::error_text(e));
        }
        Ok(values(&msgs)
            .into_iter()
            .next()
            .and_then(|r| r.into_iter().next().flatten())
            .unwrap_or_default())
    };
    let mut seen = Seen {
        show_version: one("SHOW server_version")?,
        version_num: one("SHOW server_version_num")?,
        version: one("SELECT version()")?,
        show_rupg: one("SHOW rupg.version")?,
        set_rupg: one("SET rupg.version TO '0.0.0'")?,
        ..Seen::default()
    };
    seen.status = std::mem::take(&mut c.status);
    let checks = checks(o.pin.major, &seen);
    let run = Run {
        suite: "version".into(),
        tests: checks.iter().map(|c| (c.name.to_string(), c.ok)).collect(),
        seconds: started.elapsed().as_secs(),
        ..Run::default()
    };
    Ok((run, checks))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard(n: usize) -> Vec<(String, String)> {
        (0..n).map(|i| (format!("p{i}"), "v".to_string())).collect()
    }

    fn good() -> Seen {
        let mut status = standard(14);
        status.insert(3, ("server_version".into(), "19.0 (rupg 0.1.59)".into()));
        status.push(("rupg.version".into(), "0.1.59".into()));
        Seen {
            status,
            show_version: "19.0 (rupg 0.1.59)".into(),
            version_num: "190000".into(),
            version: "PostgreSQL 19.0 (rupg 0.1.59) on x86_64-unknown-linux-gnu, compiled by rustc 1.98.0, 64-bit".into(),
            show_rupg: "0.1.59".into(),
            set_rupg: "ERROR 55P02 parameter \"rupg.version\" cannot be changed".into(),
        }
    }

    fn failed(major: u32, seen: &Seen) -> Vec<&'static str> {
        checks(major, seen).into_iter().filter(|c| !c.ok).map(|c| c.name).collect()
    }

    #[test]
    fn the_forms_of_the_spec_pass() {
        let seen = good();
        let all = checks(19, &seen);
        assert_eq!(all.len(), 8);
        assert!(all.iter().all(|c| c.ok), "{all:?}");
    }

    #[test]
    fn the_oracle_forms_fail() {
        let mut seen = good();
        seen.status[3].1 = "19beta4".into();
        seen.show_version = "19beta4".into();
        seen.version_num = "190000".into();
        seen.version =
            "PostgreSQL 19beta4 on x86_64-pc-linux-gnu, compiled by gcc 13.3.0, 64-bit".into();
        seen.status.pop();
        seen.show_rupg = String::new();
        assert_eq!(
            failed(19, &seen),
            [
                "server_version",
                "server_version_num",
                "version()",
                "rupg.version",
                "rupg.version after the standard parameters"
            ]
        );
    }

    #[test]
    fn a_shim_version_gives_its_number_and_its_parameters() {
        let mut seen = good();
        seen.status = standard(13);
        seen.status.insert(0, ("server_version".into(), "16.15 (rupg 0.1.59)".into()));
        seen.status.push(("rupg.version".into(), "0.1.59".into()));
        seen.show_version = "16.15 (rupg 0.1.59)".into();
        seen.version_num = "160015".into();
        seen.version = seen.version.replace("19.0", "16.15");
        assert!(failed(16, &seen).is_empty(), "{:?}", checks(16, &seen));
        assert_eq!(
            failed(19, &seen),
            ["server_version", "rupg.version after the standard parameters"]
        );
    }

    #[test]
    fn a_wrong_form_fails() {
        let mut seen = good();
        seen.version = "PostgreSQL 19.0 (rupg 0.1.59), compiled by rustc 1.98.0, 64-bit".into();
        seen.set_rupg = String::new();
        seen.status.push(("crdb_version".into(), "v25".into()));
        assert_eq!(
            failed(19, &seen),
            [
                "version()",
                "rupg.version is internal",
                "rupg.version after the standard parameters",
                "no parameter of another system"
            ]
        );
        assert_eq!(parse_server_version("19.0 (rupg 0.1)"), None);
        assert_eq!(parse_server_version("19 (rupg 0.1.59)"), None);
        assert_eq!(parse_server_version("19.0 (rupg 0.1.59)"), Some((19, 0, "0.1.59")));
    }
}
