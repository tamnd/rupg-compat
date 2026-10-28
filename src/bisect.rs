//! The bisect of spec/21 section 21.3.7. It builds rupg at the commits between a good commit and a bad commit, runs a SQL file against each build and the oracle, and finds the first commit where the file fails in the way that it fails at the bad commit. A search over 1,000 commits builds about 10 of them.
//!
//! When the bad commit is older than the good commit, the search finds the first commit where the file no longer fails in that way, which is the commit of a fix.

use std::fs::{self, File};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::client::Target;
use crate::oracle::{self, Oracle};
use crate::reduce::{self, Tester};

/// The result of a commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    /// The build or the start of the server failed, so the commit does not count.
    Skip(String),
    /// The file gives the answers of the oracle, or fails in another way.
    Other(Option<String>),
    /// The file fails in the way that it fails at the bad commit.
    Same,
}

/// The settings of a bisect.
pub(crate) struct Bisect<'a> {
    pub(crate) repo: PathBuf,
    pub(crate) work: PathBuf,
    pub(crate) oracle: &'a Oracle,
    pub(crate) oracle_target: Target,
    pub(crate) port: u16,
    pub(crate) timeout: Duration,
    pub(crate) statements: Vec<String>,
    pub(crate) builds: usize,
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn is_ancestor(repo: &Path, a: &str, b: &str) -> bool {
    git(repo, &["merge-base", "--is-ancestor", a, b]).is_ok()
}

/// The first commit of `commits` where `found` is true, when it is true for the last commit. A commit that gives `None` does not count.
pub(crate) fn first<F>(commits: &[String], mut found: F) -> Result<Option<String>, String>
where
    F: FnMut(&str) -> Result<Option<bool>, String>,
{
    let mut list = commits.to_vec();
    let (mut lo, mut hi) = (0, list.len().checked_sub(1).ok_or("there are no commits to search")?);
    while lo < hi {
        let mid = (lo + hi) / 2;
        match found(&list[mid])? {
            Some(true) => hi = mid,
            Some(false) => lo = mid + 1,
            None => {
                list.remove(mid);
                hi -= 1;
            }
        }
    }
    Ok(list.get(hi).cloned())
}

impl Bisect<'_> {
    fn dir(&self) -> PathBuf {
        self.work.join("bisect")
    }

    /// Builds rupg at the commit in a worktree of the work directory, and returns the binary.
    fn build(&mut self, commit: &str) -> Result<Result<PathBuf, String>, String> {
        let tree = self.dir().join("rupg");
        if tree.exists() {
            git(&tree, &["checkout", "-q", "--detach", commit])?;
        } else {
            fs::create_dir_all(self.dir()).map_err(|e| format!("{}: {e}", self.dir().display()))?;
            git(
                &self.repo,
                &["worktree", "add", "-q", "--detach", &tree.to_string_lossy(), commit],
            )?;
        }
        self.builds += 1;
        let target = self.dir().join("target");
        let log_path = self.dir().join("build.log");
        let log = File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
        let err = log.try_clone().map_err(|e| e.to_string())?;
        let status = Command::new("cargo")
            .args(["build", "--release", "--bin", "rupg"])
            .current_dir(&tree)
            .env("CARGO_TARGET_DIR", &target)
            .stdout(log)
            .stderr(err)
            .status()
            .map_err(|e| format!("cargo: {e}"))?;
        Ok(if status.success() {
            Ok(target.join("release").join("rupg"))
        } else {
            Err(format!("the build fails, see {}", log_path.display()))
        })
    }

    /// Builds the commit, starts it, and runs the file against it and the oracle.
    pub(crate) fn test(
        &mut self,
        commit: &str,
        want: Option<&str>,
    ) -> Result<(Step, Option<String>), String> {
        let bin = match self.build(commit)? {
            Ok(bin) => bin,
            Err(e) => return Ok((Step::Skip(e), None)),
        };
        let conf = self.dir().join("conf");
        let _ = fs::remove_dir_all(&conf);
        self.oracle.write_conf(&conf, self.port)?;
        let pw = self.dir().join("pw");
        fs::write(&pw, oracle::PASSWORD).map_err(|e| format!("{}: {e}", pw.display()))?;
        let log_path = self.dir().join("rupg.log");
        let log = File::create(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
        let err = log.try_clone().map_err(|e| e.to_string())?;
        let mut child = Command::new(&bin)
            .arg("serve")
            .arg("-D")
            .arg(&conf)
            .arg("--pwfile")
            .arg(&pw)
            .stdin(Stdio::null())
            .stdout(log)
            .stderr(err)
            .spawn()
            .map_err(|e| format!("{}: {e}", bin.display()))?;
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let start = Instant::now();
        let up = loop {
            if TcpStream::connect_timeout(&addr, Duration::from_secs(1)).is_ok() {
                break true;
            }
            if start.elapsed() > Duration::from_secs(60)
                || child.try_wait().ok().flatten().is_some()
            {
                break false;
            }
            std::thread::sleep(Duration::from_millis(200));
        };
        let result = if up {
            let mut server = self.oracle_target.clone();
            server.addr = addr;
            server.params.push(("rupg.compat_version".into(), self.oracle.pin.major.to_string()));
            let mut tester = Tester {
                oracle: self.oracle_target.clone(),
                server,
                timeout: self.timeout,
                runs: 0,
                max_runs: usize::MAX,
            };
            tester.signature(&self.statements).map(|sig| {
                let same = match (&sig, want) {
                    (Some(s), Some(w)) => reduce::without_number(s) == reduce::without_number(w),
                    (Some(_), None) => true,
                    _ => false,
                };
                (if same { Step::Same } else { Step::Other(sig.clone()) }, sig)
            })
        } else {
            Ok((Step::Skip(format!("the server does not start, see {}", log_path.display())), None))
        };
        let _ = child.kill();
        let _ = child.wait();
        result
    }

    /// The commits after `from` up to `to`, from the oldest.
    pub(crate) fn commits(&self, from: &str, to: &str) -> Result<Vec<String>, String> {
        let list = git(
            &self.repo,
            &["rev-list", "--reverse", "--ancestry-path", &format!("{from}..{to}")],
        )?;
        Ok(list.lines().map(str::to_string).collect())
    }

    /// The commit and its subject.
    pub(crate) fn describe(&self, commit: &str) -> String {
        git(&self.repo, &["log", "-1", "--format=%h %s", commit])
            .unwrap_or_else(|_| commit.to_string())
    }

    pub(crate) fn is_ancestor(&self, a: &str, b: &str) -> bool {
        is_ancestor(&self.repo, a, b)
    }

    /// Removes the worktree of the bisect.
    pub(crate) fn clean(&self) {
        let tree = self.dir().join("rupg");
        if tree.exists() {
            let _ = git(&self.repo, &["worktree", "remove", "--force", &tree.to_string_lossy()]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(n: usize) -> Vec<String> {
        (0..n).map(|i| i.to_string()).collect()
    }

    #[test]
    fn the_search_finds_the_first_commit() {
        for bad in 0..20 {
            let mut tests = 0;
            let got = first(&list(20), |c| {
                tests += 1;
                Ok(Some(c.parse::<usize>().unwrap() >= bad))
            })
            .unwrap();
            assert_eq!(got, Some(bad.to_string()));
            assert!(tests <= 5, "{tests}");
        }
    }

    #[test]
    fn a_skipped_commit_does_not_count() {
        let got = first(&list(10), |c| {
            let n: usize = c.parse().unwrap();
            Ok(if n == 4 { None } else { Some(n >= 4) })
        })
        .unwrap();
        assert_eq!(got, Some("5".to_string()));
    }

    #[test]
    fn a_thousand_commits_need_ten_tests() {
        let mut tests = 0;
        first(&list(1000), |c| {
            tests += 1;
            Ok(Some(c.parse::<usize>().unwrap() >= 617))
        })
        .unwrap();
        assert_eq!(tests, 10);
    }
}
