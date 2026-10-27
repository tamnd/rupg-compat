//! The resources of a run (spec/02 section 2.6.1 of tamnd/rupg): the peak memory and the CPU time of the server that the run measures.
//!
//! With `--cgroup DIR`, DIR is the cgroup v2 directory of that server. The harness reads `cpu.stat` and opens `memory.peak` when the command starts, and reads both again when it writes a result file. From Linux 6.12 a write to `memory.peak` resets the peak for the reads through the same file, so the peak is the peak of the run. On an older kernel the write fails, and the peak is the peak since the cgroup started. The result file says which.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The numbers of one run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Resources {
    /// `memory.peak` in bytes.
    pub(crate) peak: u64,
    /// `usage_usec` of `cpu.stat` at the end, less the value at the start.
    pub(crate) cpu_usec: u64,
    /// True when the peak is the peak of the run, false when it is the peak since the cgroup started.
    pub(crate) reset: bool,
}

impl Resources {
    /// The lines of a result file.
    pub(crate) fn to_toml(self) -> String {
        format!(
            "peak_memory = {}\ncpu_usec = {}\npeak_reset = {}\n",
            self.peak, self.cpu_usec, self.reset
        )
    }
}

struct Meter {
    dir: PathBuf,
    peak: File,
    reset: bool,
    cpu: u64,
}

static METER: Mutex<Option<Meter>> = Mutex::new(None);

/// Starts to measure the cgroup `dir`.
pub(crate) fn start(dir: &Path) -> Result<(), String> {
    let meter = Meter::open(dir)?;
    *METER.lock().unwrap_or_else(|e| e.into_inner()) = Some(meter);
    Ok(())
}

/// The numbers since the start or since the last lap, and a new start. None when there is no `--cgroup`.
pub(crate) fn lap() -> Result<Option<Resources>, String> {
    let mut meter = METER.lock().unwrap_or_else(|e| e.into_inner());
    let Some(m) = meter.as_mut() else {
        return Ok(None);
    };
    let r = m.read()?;
    *m = Meter::open(&m.dir)?;
    Ok(Some(r))
}

impl Meter {
    fn open(dir: &Path) -> Result<Meter, String> {
        let path = dir.join("memory.peak");
        let mut peak = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .or_else(|_| File::open(&path))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let reset = peak.write_all(b"reset\n").is_ok();
        Ok(Meter { dir: dir.to_path_buf(), peak, reset, cpu: usage(dir)? })
    }

    fn read(&mut self) -> Result<Resources, String> {
        let path = self.dir.join("memory.peak");
        let mut text = String::new();
        self.peak
            .seek(SeekFrom::Start(0))
            .and_then(|_| self.peak.read_to_string(&mut text))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let peak = text
            .trim()
            .parse()
            .map_err(|_| format!("{}: {:?} is not a number", path.display(), text.trim()))?;
        Ok(Resources {
            peak,
            cpu_usec: usage(&self.dir)?.saturating_sub(self.cpu),
            reset: self.reset,
        })
    }
}

fn usage(dir: &Path) -> Result<u64, String> {
    let path = dir.join("cpu.stat");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_usage(&text).ok_or_else(|| format!("{}: no usage_usec", path.display()))
}

fn parse_usage(text: &str) -> Option<u64> {
    text.lines().find_map(|l| l.strip_prefix("usage_usec ")).and_then(|v| v.trim().parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_comes_from_cpu_stat() {
        let text = "usage_usec 1234567\nuser_usec 1000000\nsystem_usec 234567\n";
        assert_eq!(parse_usage(text), Some(1234567));
        assert_eq!(parse_usage("user_usec 5\n"), None);
    }

    #[test]
    fn the_lines_of_a_result_file() {
        let r = Resources { peak: 10, cpu_usec: 20, reset: true };
        assert_eq!(r.to_toml(), "peak_memory = 10\ncpu_usec = 20\npeak_reset = true\n");
    }
}
