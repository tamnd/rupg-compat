//! The import of the PostgreSQL suites at the pin (spec/05 section 5.4, spec/21 section 21.4.1).
//!
//! The harness does not copy the test files of PostgreSQL into this repository. The build of each oracle has the source tree at its pin, and the import writes `corpus/postgres/N/` from it:
//!
//! - `manifest.sha256`: the SHA-256 of each file that a suite reads, in the format of `sha256sum`. A suite checks its files against the manifest before it runs, so it runs on the files of the pin or not at all.
//! - `catalogs.txt`, `system_views.txt`, `information_schema.txt` and `parameters.txt`: the lists that the catalog suite and the parameter suite run.
//! - `counts.toml`: the counts of spec/05 section 5.4, each with the rule that took it.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::scram::sha256;

pub(crate) const REGRESS: &str = "src/test/regress";
pub(crate) const ISOLATION: &str = "src/test/isolation";
pub(crate) const PIPELINE: &str = "src/test/modules/libpq_pipeline";
const SYSTEM_VIEWS: &str = "src/backend/catalog/system_views.sql";
const INFORMATION_SCHEMA: &str = "src/backend/catalog/information_schema.sql";
const GUC_PARAMETERS: &str = "src/backend/utils/misc/guc_parameters.dat";
const CATALOG_HEADERS: &str = "src/include/catalog";

/// One file that the import writes, with its text.
pub(crate) type Output = (&'static str, String);

/// One count with the rule that took it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Count {
    pub(crate) table: &'static str,
    pub(crate) key: &'static str,
    pub(crate) value: usize,
    pub(crate) rule: String,
}

fn read(src: &Path, rel: &str) -> Result<Vec<u8>, String> {
    fs::read(src.join(rel)).map_err(|e| format!("{}: {e}", src.join(rel).display()))
}

fn text(src: &Path, rel: &str) -> Result<String, String> {
    Ok(String::from_utf8_lossy(&read(src, rel)?).into_owned())
}

/// The files of a directory, as paths relative to `src`, in sorted order. Subdirectories are walked too.
fn files(src: &Path, rel: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let dir = src.join(rel);
    let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in entries {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().into_owned();
        let path = format!("{rel}/{name}");
        if e.file_type().map_err(|e| e.to_string())?.is_dir() {
            out.extend(files(src, &path)?);
        } else {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// The test names on the `test:` lines of a schedule, in order.
pub(crate) fn schedule_tests(schedule: &str) -> Vec<String> {
    schedule
        .lines()
        .filter_map(|l| l.strip_prefix("test:"))
        .flat_map(str::split_whitespace)
        .map(str::to_string)
        .collect()
}

/// An expected file with a `_N` suffix is an alternative output of a test.
fn is_alternative(file: &str) -> bool {
    let stem = file.strip_suffix(".out").unwrap_or(file);
    stem.rsplit_once('_')
        .is_some_and(|(_, n)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// The catalogs: the first argument of each `CATALOG(` line of `src/include/catalog/pg_*.h`, and whether the line has `BKI_SHARED_RELATION`.
pub(crate) fn catalogs(headers: &[(String, String)]) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for (_, text) in headers {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("CATALOG(") {
                let name = rest.split([',', ')']).next().unwrap_or("").trim().to_string();
                out.push((name, line.contains("BKI_SHARED_RELATION")));
            }
        }
    }
    out.sort();
    out
}

/// The view names of the `CREATE VIEW` and `CREATE OR REPLACE VIEW` lines of a SQL file.
pub(crate) fn views(sql: &str) -> Vec<String> {
    let mut out: Vec<String> = sql
        .lines()
        .filter_map(|l| {
            l.strip_prefix("CREATE VIEW ").or_else(|| l.strip_prefix("CREATE OR REPLACE VIEW "))
        })
        .map(|rest| {
            rest.split(|c: char| c.is_whitespace() || c == '(').next().unwrap_or("").to_string()
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The parameters of `guc_parameters.dat`: the name, the type and the context of each record.
pub(crate) fn parameters(dat: &str) -> Vec<[String; 3]> {
    let field = |record: &str, key: &str| {
        let at = record.find(&format!("{key} => '"))? + key.len() + 5;
        let len = record[at..].find('\'')?;
        Some(record[at..at + len].to_string())
    };
    let mut out: Vec<[String; 3]> = dat
        .split("{ name => '")
        .skip(1)
        .filter_map(|r| {
            let record = format!("name => '{}", r.split('}').next().unwrap_or(""));
            Some([field(&record, "name")?, field(&record, "type")?, field(&record, "context")?])
        })
        .collect();
    out.sort();
    out
}

/// The lines of a text: the number of newline bytes, as `wc -l` counts them.
fn lines(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| b == b'\n').count()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// What the import takes from one source tree.
#[derive(Debug, Default)]
pub(crate) struct Import {
    pub(crate) outputs: Vec<Output>,
    pub(crate) counts: Vec<Count>,
}

/// Reads the source tree of an oracle and makes the files of `corpus/postgres/N/`.
pub(crate) fn import(src: &Path, major: u32, commit: &str) -> Result<Import, String> {
    let mut manifest: Vec<String> = Vec::new();
    for rel in [format!("{REGRESS}/parallel_schedule"), format!("{REGRESS}/resultmap")] {
        manifest.push(rel);
    }
    // Up to 14, the tests with paths are in input/ and output/, and pg_regress makes their files when it runs.
    for dir in ["sql", "expected", "data", "input", "output"] {
        if dir == "sql" || dir == "expected" || src.join(REGRESS).join(dir).exists() {
            manifest.extend(files(src, &format!("{REGRESS}/{dir}"))?);
        }
    }
    manifest.push(format!("{ISOLATION}/isolation_schedule"));
    for dir in ["specs", "expected"] {
        manifest.extend(files(src, &format!("{ISOLATION}/{dir}"))?);
    }
    manifest.push(format!("{PIPELINE}/libpq_pipeline.c"));
    manifest.extend(files(src, &format!("{PIPELINE}/traces"))?);
    manifest.push(SYSTEM_VIEWS.into());
    manifest.push(INFORMATION_SCHEMA.into());
    let has_guc_dat = src.join(GUC_PARAMETERS).exists();
    if has_guc_dat {
        manifest.push(GUC_PARAMETERS.into());
    }
    let mut headers = Vec::new();
    for rel in files(src, CATALOG_HEADERS)? {
        let name = rel.rsplit('/').next().unwrap_or("");
        if name.starts_with("pg_") && name.ends_with(".h") {
            let t = text(src, &rel)?;
            if t.lines().any(|l| l.starts_with("CATALOG(")) {
                manifest.push(rel.clone());
                headers.push((rel, t));
            }
        }
    }
    manifest.sort();
    let mut sums = String::new();
    for rel in &manifest {
        sums.push_str(&format!("{}  {rel}\n", hex(&sha256(&read(src, rel)?))));
    }

    let mut counts = Vec::new();
    let mut count = |table, key, value, rule| {
        counts.push(Count { table, key, value, rule: String::from(rule) })
    };

    // The regression suite.
    let scheduled = schedule_tests(&text(src, &format!("{REGRESS}/parallel_schedule"))?);
    let sql = files(src, &format!("{REGRESS}/sql"))?;
    let expected = files(src, &format!("{REGRESS}/expected"))?;
    let name = |rel: &str| rel.rsplit('/').next().unwrap_or("").to_string();
    let primary: Vec<&String> = expected.iter().filter(|r| !is_alternative(&name(r))).collect();
    let mut primary_lines = 0;
    let (mut error_lines, mut error_files) = (0, 0);
    for rel in &primary {
        let bytes = read(src, rel)?;
        primary_lines += lines(&bytes);
        let errors = bytes.split(|&b| b == b'\n').filter(|l| l.starts_with(b"ERROR:")).count();
        error_lines += errors;
        error_files += usize::from(errors > 0);
    }
    let mut scheduled_lines = 0;
    let (mut scheduled_error_lines, mut scheduled_error_files) = (0, 0);
    for t in &scheduled {
        let mut rel = format!("{REGRESS}/expected/{t}.out");
        if !src.join(&rel).exists() {
            rel = format!("{REGRESS}/output/{t}.source");
        }
        let bytes = read(src, &rel)?;
        scheduled_lines += lines(&bytes);
        let errors = bytes.split(|&b| b == b'\n').filter(|l| l.starts_with(b"ERROR:")).count();
        scheduled_error_lines += errors;
        scheduled_error_files += usize::from(errors > 0);
    }
    count(
        "regress",
        "scheduled_tests",
        scheduled.len(),
        "the names on the test: lines of parallel_schedule",
    );
    count(
        "regress",
        "sql_files",
        sql.iter().filter(|r| r.ends_with(".sql")).count(),
        "the files sql/*.sql",
    );
    count("regress", "expected_files", expected.len(), "the files in expected/");
    count(
        "regress",
        "primary_expected_files",
        primary.len(),
        "the files in expected/ without an alternative suffix _N",
    );
    count("regress", "expected_lines", primary_lines, "the lines of the primary expected files");
    count(
        "regress",
        "error_lines",
        error_lines,
        "the lines of the primary expected files that start with ERROR:",
    );
    count("regress", "error_files", error_files, "the primary expected files with an ERROR: line");
    count(
        "regress",
        "scheduled_expected_lines",
        scheduled_lines,
        "the lines of expected/T.out, or of output/T.source up to 14, for each scheduled test T",
    );
    count(
        "regress",
        "scheduled_error_lines",
        scheduled_error_lines,
        "the lines of those files that start with ERROR:",
    );
    count(
        "regress",
        "scheduled_error_files",
        scheduled_error_files,
        "those files with an ERROR: line",
    );

    // The isolation suite and libpq_pipeline.
    let specs = files(src, &format!("{ISOLATION}/specs"))?;
    let iso_scheduled = schedule_tests(&text(src, &format!("{ISOLATION}/isolation_schedule"))?);
    count(
        "isolation",
        "specs",
        specs.iter().filter(|r| r.ends_with(".spec")).count(),
        "the files specs/*.spec",
    );
    count(
        "isolation",
        "scheduled_specs",
        iso_scheduled.len(),
        "the names on the test: lines of isolation_schedule",
    );
    count(
        "isolation",
        "expected_files",
        files(src, &format!("{ISOLATION}/expected"))?.len(),
        "the files in expected/",
    );
    let traces = files(src, &format!("{PIPELINE}/traces"))?;
    count(
        "libpq_pipeline",
        "traces",
        traces.iter().filter(|r| r.ends_with(".trace")).count(),
        "the files traces/*.trace",
    );

    // The lists of the catalog suite and the parameter suite.
    let cats = catalogs(&headers);
    let sysviews = views(&text(src, SYSTEM_VIEWS)?);
    let infoviews = views(&text(src, INFORMATION_SCHEMA)?);
    count("catalog", "catalogs", cats.len(), "the CATALOG( lines of src/include/catalog/pg_*.h");
    count(
        "catalog",
        "shared_catalogs",
        cats.iter().filter(|c| c.1).count(),
        "the CATALOG( lines with BKI_SHARED_RELATION",
    );
    count(
        "catalog",
        "system_views",
        sysviews.len(),
        "the CREATE VIEW and CREATE OR REPLACE VIEW lines of system_views.sql",
    );
    count(
        "catalog",
        "statistics_views",
        sysviews.iter().filter(|v| v.starts_with("pg_stat")).count(),
        "the system views whose name starts with pg_stat, which includes pg_statio",
    );
    count(
        "catalog",
        "information_schema_views",
        infoviews.len(),
        "the CREATE VIEW lines of information_schema.sql",
    );
    let mut outputs: Vec<Output> = vec![
        ("manifest.sha256", sums),
        (
            "catalogs.txt",
            cats.iter()
                .map(|(n, s)| format!("{n} {}\n", if *s { "shared" } else { "local" }))
                .collect(),
        ),
        ("system_views.txt", sysviews.iter().map(|v| format!("{v}\n")).collect()),
        ("information_schema.txt", infoviews.iter().map(|v| format!("{v}\n")).collect()),
    ];
    if has_guc_dat {
        let params = parameters(&text(src, GUC_PARAMETERS)?);
        count("parameters", "parameters", params.len(), "the records of guc_parameters.dat");
        let mut by_type: BTreeMap<&str, usize> = BTreeMap::new();
        for p in &params {
            *by_type.entry(p[1].as_str()).or_default() += 1;
        }
        for (ty, key) in [
            ("int", "int"),
            ("bool", "bool"),
            ("string", "string"),
            ("enum", "enum"),
            ("real", "real"),
        ] {
            counts.push(Count {
                table: "parameters",
                key,
                value: by_type.get(ty).copied().unwrap_or(0),
                rule: format!("the records of guc_parameters.dat with type => '{ty}'"),
            });
        }
        outputs.push((
            "parameters.txt",
            params.iter().map(|p| format!("{} {} {}\n", p[0], p[1], p[2])).collect(),
        ));
    }
    let mut toml = format!(
        "# The counts of spec/05 section 5.4, taken by `rupg-compat import` from the source tree of the oracle.\n# Each count has the rule that took it. `rupg-compat import --check` takes them again.\nversion = {major}\ncommit = \"{commit}\"\n"
    );
    let mut table = "";
    for c in &counts {
        if c.table != table {
            toml.push_str(&format!("\n[{}]\n", c.table));
            table = c.table;
        }
        toml.push_str(&format!("# {}\n{} = {}\n", c.rule, c.key, c.value));
    }
    outputs.push(("counts.toml", toml));
    Ok(Import { outputs, counts })
}

/// Checks the files under `prefix` in the source tree against the manifest. Returns the number of files that it checked.
pub(crate) fn verify(src: &Path, dir: &Path, prefix: &str) -> Result<usize, String> {
    let path = dir.join("manifest.sha256");
    let manifest = fs::read_to_string(&path)
        .map_err(|e| format!("{}: {e}; run rupg-compat import first", path.display()))?;
    let mut checked = 0;
    for line in manifest.lines() {
        let (sum, rel) =
            line.split_once("  ").ok_or_else(|| format!("{}: bad line {line}", path.display()))?;
        if !rel.starts_with(prefix) {
            continue;
        }
        if hex(&sha256(&read(src, rel)?)) != sum {
            return Err(format!("{rel} is not the file of the pin in {}", path.display()));
        }
        checked += 1;
    }
    if checked == 0 {
        return Err(format!("{} has no file under {prefix}", path.display()));
    }
    Ok(checked)
}

/// The directory of the import of version N.
pub(crate) fn corpus_dir(corpus: &Path, major: u32) -> PathBuf {
    corpus.join("postgres").join(major.to_string())
}

/// The files of `dir` that differ from the import, or that are missing.
pub(crate) fn check(dir: &Path, import: &Import) -> Vec<String> {
    import
        .outputs
        .iter()
        .filter(|(name, text)| fs::read_to_string(dir.join(name)).ok().as_deref() != Some(text))
        .map(|(name, _)| name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedules_list_the_tests_in_order() {
        let s = "# comment\ntest: a b\n\n# x\ntest: c\nignore: d\n";
        assert_eq!(schedule_tests(s), ["a", "b", "c"]);
    }

    #[test]
    fn alternatives_have_a_number_suffix() {
        assert!(is_alternative("int8_1.out"));
        assert!(!is_alternative("int8.out"));
        assert!(!is_alternative("float4-misrounded-input.out"));
        assert!(!is_alternative("create_index_spgist.out"));
    }

    #[test]
    fn catalogs_views_and_parameters_are_found() {
        let h = vec![(
            "pg_database.h".to_string(),
            "/* x */\nCATALOG(pg_database,1262,DatabaseRelationId) BKI_SHARED_RELATION BKI_ROWTYPE_OID(1248,X)\n{\n".to_string(),
        )];
        assert_eq!(catalogs(&h), [("pg_database".to_string(), true)]);
        let sql = "CREATE VIEW pg_roles AS\nCREATE OR REPLACE VIEW pg_stat_io(a) AS\n  CREATE VIEW indented AS\n";
        assert_eq!(views(sql), ["pg_roles", "pg_stat_io"]);
        let dat = "[\n{ name => 'work_mem', type => 'int', context => 'PGC_USERSET', group => 'X',\n  boot_val => '4096' },\n{ name => 'fsync', type => 'bool', context => 'PGC_SIGHUP' },\n]";
        let p = parameters(dat);
        assert_eq!(p.len(), 2);
        assert_eq!(p[1], ["work_mem".to_string(), "int".into(), "PGC_USERSET".into()]);
    }

    #[test]
    fn a_small_tree_is_imported_and_checked() {
        let src = std::env::temp_dir().join(format!("rupg-compat-import-{}", std::process::id()));
        let put = |rel: &str, text: &str| {
            let path = src.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        };
        put(&format!("{REGRESS}/parallel_schedule"), "test: a b\n");
        put(&format!("{REGRESS}/resultmap"), "");
        put(&format!("{REGRESS}/sql/a.sql"), "SELECT 1;\n");
        put(&format!("{REGRESS}/sql/b.sql"), "SELECT x;\n");
        put(&format!("{REGRESS}/expected/a.out"), "SELECT 1;\n 1\n");
        put(
            &format!("{REGRESS}/expected/b.out"),
            "SELECT x;\nERROR:  column \"x\" does not exist\n",
        );
        put(&format!("{REGRESS}/expected/b_1.out"), "SELECT x;\nERROR:  other\n");
        put(&format!("{ISOLATION}/isolation_schedule"), "test: s\n");
        put(&format!("{ISOLATION}/specs/s.spec"), "session s1\n");
        put(&format!("{ISOLATION}/expected/s.out"), "");
        put(&format!("{PIPELINE}/libpq_pipeline.c"), "");
        put(&format!("{PIPELINE}/traces/simple.trace"), "");
        put(SYSTEM_VIEWS, "CREATE VIEW pg_roles AS\n");
        put(INFORMATION_SCHEMA, "CREATE VIEW tables AS\n");
        put(
            &format!("{CATALOG_HEADERS}/pg_class.h"),
            "CATALOG(pg_class,1259,RelationRelationId)\n",
        );
        let imp = import(&src, 19, "abc").unwrap();
        let get = |key: &str| imp.counts.iter().find(|c| c.key == key).unwrap().value;
        assert_eq!(get("scheduled_tests"), 2);
        assert_eq!(get("expected_files"), 3);
        assert_eq!(get("primary_expected_files"), 2);
        assert_eq!(get("expected_lines"), 4);
        assert_eq!((get("error_lines"), get("error_files")), (1, 1));
        assert_eq!((get("specs"), get("traces"), get("catalogs")), (1, 1, 1));
        assert_eq!(imp.outputs[0].1.lines().count(), 15);
        let dir = src.join("corpus");
        fs::create_dir_all(&dir).unwrap();
        for (name, text) in &imp.outputs {
            fs::write(dir.join(name), text).unwrap();
        }
        assert!(check(&dir, &imp).is_empty());
        put(&format!("{REGRESS}/expected/a.out"), "changed\n");
        assert_eq!(
            check(&dir, &import(&src, 19, "abc").unwrap()),
            ["manifest.sha256", "counts.toml"]
        );
        fs::remove_dir_all(&src).unwrap();
    }

    #[test]
    fn lines_count_newlines() {
        assert_eq!(lines(b"a\nb\n"), 2);
        assert_eq!(lines(b"a\nb"), 1);
    }
}
