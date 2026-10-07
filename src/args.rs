//! The command line: positional words and `--name value` options.
//!
//! The harness has few dependencies, so it does not use an argument crate. Every option takes a value, except the names in `FLAGS`. Everything after `--` is positional.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// The options that take no value.
const FLAGS: [&str; 5] = ["--all", "--check", "--force", "--help", "--keep"];

/// A parsed command line.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Args {
    pub(crate) words: Vec<String>,
    options: BTreeMap<String, String>,
    /// The words after `--`, for a command that runs another program.
    pub(crate) rest: Vec<String>,
}

impl Args {
    pub(crate) fn parse(raw: impl IntoIterator<Item = String>) -> Result<Args, String> {
        let mut args = Args::default();
        let mut raw = raw.into_iter();
        while let Some(word) = raw.next() {
            if word == "--" {
                args.rest = raw.by_ref().collect();
            } else if let Some(name) = word.strip_prefix("--") {
                let (name, value) = match name.split_once('=') {
                    Some((n, v)) => (n.to_string(), v.to_string()),
                    None if FLAGS.contains(&word.as_str()) => (name.to_string(), String::new()),
                    None => {
                        let value = raw.next().ok_or_else(|| format!("{word} needs a value"))?;
                        (name.to_string(), value)
                    }
                };
                if args.options.insert(name, value).is_some() {
                    return Err(format!("{word} is given twice"));
                }
            } else {
                args.words.push(word);
            }
        }
        Ok(args)
    }

    pub(crate) fn word(&self, i: usize) -> Option<&str> {
        self.words.get(i).map(String::as_str)
    }

    pub(crate) fn flag(&self, name: &str) -> bool {
        self.options.contains_key(name)
    }

    pub(crate) fn get(&self, name: &str) -> Option<&str> {
        self.options.get(name).map(String::as_str)
    }

    pub(crate) fn number<T: std::str::FromStr>(&self, name: &str) -> Result<Option<T>, String> {
        self.get(name)
            .map(|v| v.parse().map_err(|_| format!("--{name} {v} is not a number")))
            .transpose()
    }

    /// `--compat-version`, or the reference version.
    pub(crate) fn compat_version(&self, reference: u32) -> Result<u32, String> {
        Ok(self.number("compat-version")?.unwrap_or(reference))
    }

    /// The work directory: `--work`, then `RUPG_COMPAT_WORK`, then `work` in the current directory.
    pub(crate) fn work_dir(&self) -> PathBuf {
        let dir = self
            .get("work")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("RUPG_COMPAT_WORK").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("work"));
        std::path::absolute(&dir).unwrap_or(dir)
    }

    /// Fails if an option is not in `known` or if there are more than `words` positional words, so that a typo does not pass in silence.
    /// For example, `oracle stop 14` fails. The version comes from `--compat-version 14`.
    pub(crate) fn only(&self, words: usize, known: &[&str]) -> Result<(), String> {
        if let Some(w) = self.words.get(words) {
            return Err(format!("unexpected word {w:?}"));
        }
        match self.options.keys().find(|k| !known.contains(&k.as_str()) && *k != "work") {
            Some(k) => Err(format!("unknown option --{k}")),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<Args, String> {
        Args::parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn words_options_flags_and_rest() {
        let a = parse("oracle build --compat-version 18 --all --jobs=4 -- psql -c x").unwrap();
        assert_eq!(a.words, ["oracle", "build"]);
        assert_eq!(a.compat_version(19), Ok(18));
        assert!(a.flag("all"));
        assert_eq!(a.number::<u32>("jobs"), Ok(Some(4)));
        assert_eq!(a.rest, ["psql", "-c", "x"]);
        assert_eq!(parse("x").unwrap().compat_version(19), Ok(19));
    }

    #[test]
    fn errors() {
        assert!(parse("--compat-version").is_err());
        assert!(parse("--jobs 1 --jobs 2").is_err());
        assert!(parse("--compat-version x").unwrap().compat_version(19).is_err());
        assert!(parse("--colour red").unwrap().only(1, &["compat-version"]).is_err());
        assert!(parse("oracle stop 14").unwrap().only(2, &[]).is_err());
        assert!(
            parse("oracle stop --compat-version 14").unwrap().only(2, &["compat-version"]).is_ok()
        );
    }
}
