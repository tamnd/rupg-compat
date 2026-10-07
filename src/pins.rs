//! The pins of `pins.toml`: the PostgreSQL commit of each oracle, the client versions and the rupg commit.
//!
//! Version 19 is the reference for rupg. Versions 14 to 18 are the oracles for the shims of `rupg.compat_version`. See spec/21 section 21.3.3 and spec/05 section 5.6.5 of tamnd/rupg.

use std::fmt;

use crate::toml::{self, Table, Value};

/// The text of `pins.toml`, built into the binary so that it runs from any directory.
pub(crate) const PINS_TOML: &str = include_str!("../pins.toml");

/// The pin of one oracle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pin {
    /// The major version.
    pub(crate) major: u32,
    /// The release that the pin gives, for example `18.6` or `19beta4`.
    pub(crate) release: String,
    /// The git branch or tag in the PostgreSQL repository.
    pub(crate) git_ref: String,
    /// The full commit hash. The build checks out exactly this commit.
    pub(crate) commit: String,
}

/// The pin of one client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Client {
    pub(crate) name: String,
    pub(crate) version: String,
}

/// Every pin of the harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Pins {
    /// The PostgreSQL repository that the oracles are built from.
    pub(crate) repository: String,
    /// The major version of the reference oracle.
    pub(crate) reference: u32,
    /// The oracle pins, oldest first.
    pub(crate) postgres: Vec<Pin>,
    /// The rupg commit under test.
    pub(crate) rupg_commit: String,
    /// The client pins, in name order.
    pub(crate) clients: Vec<Client>,
}

/// An error in `pins.toml`.
#[derive(Debug)]
pub(crate) struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pins.toml: {}", self.0)
    }
}

impl Pins {
    /// The pins that are built into the binary.
    pub(crate) fn builtin() -> Pins {
        Pins::parse(PINS_TOML).unwrap_or_else(|e| panic!("{e}"))
    }

    /// Reads the pins from the text of a `pins.toml` file.
    pub(crate) fn parse(text: &str) -> Result<Pins, Error> {
        let root = toml::parse(text).map_err(|e| Error(e.to_string()))?;
        let pg = table(&root, "postgres")?;
        let mut postgres = Vec::new();
        for (key, value) in pg {
            let Value::Table(t) = value else { continue };
            let major: u32 = key
                .parse()
                .map_err(|_| Error(format!("[postgres.{key}] is not a major version")))?;
            let pin = Pin {
                major,
                release: string(t, "release", key)?,
                git_ref: string(t, "ref", key)?,
                commit: string(t, "commit", key)?,
            };
            if pin.commit.len() != 40 || !pin.commit.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(Error(format!("[postgres.{key}] commit is not a full commit hash")));
            }
            postgres.push(pin);
        }
        postgres.sort_by_key(|p| p.major);
        let reference = pg
            .get("reference")
            .and_then(Value::as_int)
            .and_then(|i| u32::try_from(i).ok())
            .ok_or_else(|| Error("[postgres] has no reference".into()))?;
        if !postgres.iter().any(|p| p.major == reference) {
            return Err(Error(format!("the reference {reference} has no pin")));
        }
        let rupg = table(&root, "rupg")?;
        let mut clients = Vec::new();
        if let Some(Value::Table(c)) = root.get("clients") {
            for (name, value) in c {
                let t = value
                    .as_table()
                    .ok_or_else(|| Error(format!("client {name} is not a table")))?;
                clients.push(Client { name: name.clone(), version: string(t, "version", name)? });
            }
        }
        Ok(Pins {
            repository: string(pg, "repository", "postgres")?,
            reference,
            postgres,
            rupg_commit: string(rupg, "commit", "rupg")?,
            clients,
        })
    }
}

fn table<'a>(root: &'a Table, name: &str) -> Result<&'a Table, Error> {
    root.get(name).and_then(Value::as_table).ok_or_else(|| Error(format!("no [{name}] table")))
}

fn string(t: &Table, key: &str, owner: &str) -> Result<String, Error> {
    t.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error(format!("[{owner}] has no string `{key}`")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_parses() {
        let pins = Pins::builtin();
        assert_eq!(pins.reference, 19);
        assert_eq!(pins.rupg_commit.len(), 40);
    }

    #[test]
    fn one_pin_for_each_major_version_from_14_to_19() {
        let majors: Vec<u32> = Pins::builtin().postgres.iter().map(|p| p.major).collect();
        assert_eq!(majors, [14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn each_release_starts_with_its_major_version() {
        for pin in Pins::builtin().postgres {
            assert!(pin.release.starts_with(&pin.major.to_string()), "{pin:?}");
        }
    }

    #[test]
    fn each_client_has_a_directory_with_its_version() {
        let pins = Pins::builtin();
        let reference = pins.postgres.iter().find(|p| p.major == pins.reference).unwrap();
        let mut names = Vec::new();
        for c in &pins.clients {
            let dir = std::path::Path::new("clients").join(&c.name);
            assert!(dir.join("run.sh").exists(), "{} has no run.sh", c.name);
            // libpq, psql and pg_dump come with the oracle build.
            if ["libpq", "psql", "pg_dump"].contains(&c.name.as_str()) {
                assert_eq!(c.version, reference.release);
            } else {
                let install = [
                    "run.sh",
                    "go.mod",
                    "Cargo.toml",
                    "package.json",
                    "pom.xml",
                    "scenario.csproj",
                    "Gemfile",
                    "mix.exs",
                ]
                .iter()
                .filter_map(|f| std::fs::read_to_string(dir.join(f)).ok())
                .collect::<String>();
                assert!(install.contains(&c.version), "{} does not install {}", c.name, c.version);
            }
            names.push(c.name.clone());
        }
        let mut dirs: Vec<String> = std::fs::read_dir("clients")
            .unwrap()
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        assert_eq!(dirs, names);
    }

    #[test]
    fn the_shims_are_on_their_release_tags() {
        for pin in Pins::builtin().postgres.iter().filter(|p| p.major != 19) {
            assert_eq!(pin.git_ref, format!("REL_{}", pin.release.replace('.', "_")));
        }
    }

    #[test]
    fn a_short_commit_is_an_error() {
        let text = PINS_TOML.replace("7d3d2db7d5e533fa42a4202c7d308cbb0cbf9333", "7d3d2db7");
        assert!(Pins::parse(&text).is_err());
    }
}
