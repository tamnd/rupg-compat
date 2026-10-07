//! The comparison of the answers of two servers (spec/05 sections 5.2 and 5.3, spec/21 section 21.3.6).
//!
//! The comparison works on the backend messages of each connection. It cuts them into groups that end with `ReadyForQuery` and compares group `i` of one side with group `i` of the other. The frontend messages of the group give its statements, and the statements decide some of the rules, for example the row order and the time columns.
//!
//! Each rule of section 5.2 and each excluded item of section 5.3 has a counter. `applied` counts the values that a rule took out of the comparison. `differed` counts those that were not equal on the two sides. Item 14, the server log, is never compared, so its counters stay at zero.

use std::collections::{BTreeMap, BTreeSet};

use crate::message::{Dir, Msg, Val};
use crate::trace::{Event, Line, format_msg};

/// The closed list of spec/05 section 5.3, in its order.
pub(crate) const ITEMS: [&str; 15] = [
    "EXPLAIN output",
    "values of user OIDs",
    "values of statistics counters",
    "physical row identity (ctid, xmin, xmax, cmin, cmax)",
    "sizes",
    "C functions and LOAD",
    "physical streaming replication",
    "data directory files and WAL positions",
    "time",
    "platform text of version() and the F, L, R error fields",
    "pg_node_tree and TOAST relations",
    "suffix of server_version",
    "process ID",
    "server log lines",
    "file system paths",
];

/// The rules of section 5.2 that are not items of the closed list.
pub(crate) const RULES: [&str; 4] = [
    "row order without ORDER BY (multiset)",
    "CopyData boundaries (joined)",
    "BackendKeyData secret key (length only)",
    "SCRAM nonces, salt, proof and signature",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Count {
    pub(crate) applied: usize,
    pub(crate) differed: usize,
}

/// Where a difference is: the connection and the group in it.
pub(crate) type Place = (u32, usize);

#[derive(Debug, Default)]
pub(crate) struct Outcome {
    /// The groups that the comparison looked at, and the backend messages in them.
    pub(crate) groups: usize,
    pub(crate) messages: usize,
    pub(crate) items: [Count; 15],
    pub(crate) rules: [Count; 4],
    /// The first difference of each group that differs.
    pub(crate) diffs: BTreeMap<Place, String>,
    /// The groups that the unstable filter took out.
    pub(crate) unstable: usize,
}

impl Outcome {
    pub(crate) fn add(&mut self, other: &Outcome) {
        self.groups += other.groups;
        self.messages += other.messages;
        self.unstable += other.unstable;
        for (a, b) in self.items.iter_mut().zip(other.items) {
            a.applied += b.applied;
            a.differed += b.differed;
        }
        for (a, b) in self.rules.iter_mut().zip(other.rules) {
            a.applied += b.applied;
            a.differed += b.differed;
        }
    }
}

/// One group of one connection: its frontend and backend messages.
#[derive(Debug, Default)]
struct Group<'a> {
    front: Vec<&'a Msg>,
    back: Vec<&'a Msg>,
}

/// Cuts the lines of each connection into groups that end with `ReadyForQuery`.
fn groups(lines: &[Line]) -> BTreeMap<u32, Vec<Group<'_>>> {
    let mut out: BTreeMap<u32, Vec<Group<'_>>> = BTreeMap::new();
    for l in lines {
        let Event::Msg(m) = &l.event else { continue };
        let gs = out.entry(l.conn).or_default();
        if gs.is_empty() {
            gs.push(Group::default());
        }
        let g = gs.last_mut().expect("one group at least");
        if l.dir == Dir::F {
            g.front.push(m);
        } else {
            g.back.push(m);
            if m.name() == "ReadyForQuery" {
                gs.push(Group::default());
            }
        }
    }
    for gs in out.values_mut() {
        if gs.last().is_some_and(|g| g.front.is_empty() && g.back.is_empty()) {
            gs.pop();
        }
    }
    out
}

/// What the statements of a group do, as far as the rules care.
#[derive(Debug, Default)]
struct Context {
    explain: bool,
    language_c: bool,
    statistics: bool,
    sizes: bool,
    time: bool,
    files: bool,
    version: bool,
    paths: bool,
    ordered: bool,
    replication: bool,
}

const TIME_FUNCTIONS: [&str; 10] = [
    "now()",
    "current_timestamp",
    "clock_timestamp",
    "statement_timestamp",
    "transaction_timestamp",
    "timeofday",
    "localtimestamp",
    "current_time",
    "localtime",
    "current_date",
];

/// The functions whose answers are files of the data directory or WAL positions (item 8). A `pg_lsn` value is out by its type. `pg_wal_lsn_diff` gives a number, so it is in the list. The list names functions, not the text `wal`, because many parameters, such as `max_wal_size`, have it in their names.
const FILE_FUNCTIONS: [&str; 11] = [
    "pg_ls_dir",
    "pg_read_file",
    "pg_read_binary_file",
    "pg_stat_file",
    "pg_ls_waldir",
    "pg_ls_logdir",
    "pg_ls_tmpdir",
    "pg_ls_archive_statusdir",
    "pg_walfile_name",
    "pg_split_walfile_name",
    "pg_wal_lsn_diff",
];

const PATH_SETTINGS: [&str; 4] = ["data_directory", "config_file", "hba_file", "ident_file"];

/// True when a statement says `LANGUAGE C` or `LANGUAGE 'c'`. The text is in lower case.
fn language_c(text: &str) -> bool {
    text.match_indices("language").any(|(i, w)| {
        let rest = text[i + w.len()..].trim_start();
        let after = |n: usize| {
            rest[n..].chars().next().is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
        };
        rest.starts_with("'c'") || rest.starts_with('c') && after(1)
    })
}

impl Context {
    fn new(sql: String, replication: bool) -> Context {
        let s = sql.to_ascii_lowercase();
        let first = s.split_whitespace().next().unwrap_or("").to_string();
        Context {
            explain: s.split(';').any(|st| st.trim_start().starts_with("explain")),
            language_c: language_c(&s)
                || s.split(';').any(|st| st.trim_start().starts_with("load ")),
            statistics: s.contains("pg_stat"),
            sizes: s.contains("_size(") || s.contains("pg_size_pretty"),
            time: TIME_FUNCTIONS.iter().any(|f| s.contains(f)),
            files: FILE_FUNCTIONS.iter().any(|f| s.contains(f)),
            version: s.contains("version()"),
            paths: PATH_SETTINGS.iter().any(|p| s.contains(p)),
            ordered: s.contains("order by") || first == "fetch",
            replication,
        }
    }
}

/// The statements of a group: the text of each `Query` and of each `Parse`, and of the prepared statement that each `Bind` names.
fn context(
    group: &Group<'_>,
    prepared: &mut BTreeMap<Vec<u8>, String>,
    replication: bool,
) -> Context {
    let mut sql = String::new();
    for m in &group.front {
        let text = |i: usize| m.vals.get(i).and_then(Val::bytes).unwrap_or_default();
        match m.name() {
            "Query" => sql.push_str(&String::from_utf8_lossy(text(0))),
            "Parse" => {
                let s = String::from_utf8_lossy(text(1)).into_owned();
                prepared.insert(text(0).to_vec(), s.clone());
                sql.push_str(&s);
            }
            "Bind" => {
                if let Some(s) = prepared.get(text(1)) {
                    sql.push_str(s);
                }
            }
            _ => continue,
        }
        sql.push_str(";\n");
    }
    Context::new(sql, replication)
}

/// A value that a rule took out, with the place where it was.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Mask {
    slot: usize,
    rule: Rule,
    orig: Val,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    /// An item of the closed list, numbered from 1.
    Item(usize),
    /// A rule of section 5.2, an index of `RULES`.
    Rule(usize),
}

/// One message after the rules, with the values that the rules took out.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Shown {
    msg: Msg,
    masks: Vec<Mask>,
}

fn excluded(rule: Rule) -> Val {
    Val::Str(
        match rule {
            Rule::Item(n) => format!("<excluded {n}>"),
            Rule::Rule(n) => format!("<rule {}>", n + 1),
        }
        .into_bytes(),
    )
}

impl Shown {
    fn mask(&mut self, slot: usize, rule: Rule, v: &mut Val) {
        let orig = std::mem::replace(v, excluded(rule));
        self.masks.push(Mask { slot, rule, orig });
    }
}

/// The column names and types of the last `RowDescription`.
#[derive(Clone, Debug, Default)]
struct Columns {
    names: Vec<String>,
    types: Vec<i64>,
}

impl Columns {
    fn of(msg: &Msg) -> Columns {
        let fields = msg.vals[0].list();
        Columns {
            names: fields
                .iter()
                .map(|f| {
                    String::from_utf8_lossy(
                        f.list().first().and_then(Val::bytes).unwrap_or_default(),
                    )
                    .into_owned()
                })
                .collect(),
            types: fields.iter().map(|f| f.list().get(3).and_then(Val::int).unwrap_or(0)).collect(),
        }
    }
}

const NUMERIC_TYPES: [i64; 7] = [20, 21, 23, 26, 700, 701, 1700];
const TIME_TYPES: [i64; 6] = [1082, 1083, 1114, 1184, 1186, 1266];
const PID_COLUMNS: [&str; 3] = ["pid", "leader_pid", "pg_backend_pid"];
const ROW_IDENTITY: [&str; 5] = ["ctid", "xmin", "xmax", "cmin", "cmax"];

/// The rule that takes out column `c` of a `DataRow`, if one does.
fn column_rule(ctx: &Context, cols: &Columns, c: usize, row: &[Val]) -> Option<usize> {
    let name = cols.names.get(c).map(String::as_str).unwrap_or("");
    let ty = cols.types.get(c).copied().unwrap_or(0);
    if [27, 28, 29, 5069].contains(&ty) || ROW_IDENTITY.contains(&name) {
        return Some(4);
    }
    if ty == 194 || name == "reltoastrelid" {
        return Some(11);
    }
    // Item 2: an OID at or above FirstNormalObjectId (16384) names a user object.
    if ty == 26
        && row
            .get(c)
            .and_then(Val::bytes)
            .and_then(|b| std::str::from_utf8(b).ok()?.parse::<u64>().ok())
            .is_some_and(|oid| oid >= 16_384)
    {
        return Some(2);
    }
    if ty == 3220 || ctx.files {
        return Some(8);
    }
    if PID_COLUMNS.contains(&name) {
        return Some(13);
    }
    if ["sourcefile", "file_name"].contains(&name) || PATH_SETTINGS.contains(&name) {
        return Some(15);
    }
    if ctx.paths && ["setting", "reset_val", "boot_val", "current_setting"].contains(&name) {
        let names = cols.names.iter().position(|n| n == "name");
        let row_name = names
            .and_then(|i| row.get(i)?.bytes())
            .map(|b| String::from_utf8_lossy(b).into_owned());
        if name == "current_setting"
            || row_name.is_some_and(|n| PATH_SETTINGS.contains(&n.as_str()))
        {
            return Some(15);
        }
    }
    if ["relpages", "relallvisible", "relallfrozen"].contains(&name)
        || ctx.sizes && (NUMERIC_TYPES.contains(&ty) || name.contains("size"))
    {
        return Some(5);
    }
    // Item 3: the wait event of a process is a sample of its state, like a counter.
    if name == "reltuples"
        || ctx.statistics
            && (NUMERIC_TYPES.contains(&ty)
                || TIME_TYPES.contains(&ty)
                || ["wait_event_type", "wait_event"].contains(&name))
    {
        return Some(3);
    }
    if ctx.time && TIME_TYPES.contains(&ty) {
        return Some(9);
    }
    None
}

/// True when column `c` of a row is the value of `server_version`: the column of `SHOW server_version`, or a value column of the `server_version` row of `SHOW ALL` or `pg_settings`.
fn server_version_value(cols: &Columns, c: usize, row: &[Val]) -> bool {
    let name = cols.names.get(c).map(String::as_str).unwrap_or("");
    if name == "server_version" {
        return true;
    }
    let row_name = cols.names.iter().position(|n| n == "name").and_then(|i| row.get(i)?.bytes());
    row_name == Some(b"server_version") && ["setting", "reset_val", "boot_val"].contains(&name)
}

/// Applies the rules to the backend messages of one group.
fn show(group: &Group<'_>, ctx: &Context, cols: &mut Columns) -> Vec<Shown> {
    let mut out: Vec<Shown> = Vec::new();
    for &m in &group.back {
        let mut s = Shown { msg: m.clone(), masks: Vec::new() };
        if ctx.replication {
            let mut v = Val::Str(format_msg(m).into_bytes());
            s.msg = Msg::new(Dir::B, m.name(), Vec::new());
            s.mask(0, Rule::Item(7), &mut v);
            out.push(s);
            continue;
        }
        if ctx.language_c {
            let mut v = Val::Str(format_msg(m).into_bytes());
            s.msg = Msg::new(Dir::B, "Message", Vec::new());
            s.mask(0, Rule::Item(6), &mut v);
            out.push(s);
            continue;
        }
        let mut vals = std::mem::take(&mut s.msg.vals);
        match m.name() {
            "RowDescription" => {
                *cols = Columns::of(m);
                // Section 5.2: the value of a table OID that is not zero is not compared.
                if let Val::List(fields) = &mut vals[0] {
                    for (i, f) in fields.iter_mut().enumerate() {
                        if let Val::Tuple(v) = f
                            && v[1] != Val::Int(0)
                        {
                            s.mask(i, Rule::Item(2), &mut v[1]);
                        }
                    }
                }
            }
            "DataRow" if ctx.explain => {
                let mut v = std::mem::replace(&mut vals[0], Val::List(Vec::new()));
                s.mask(0, Rule::Item(1), &mut v);
            }
            "DataRow" => {
                let row = vals[0].list().to_vec();
                if let Val::List(values) = &mut vals[0] {
                    for (c, v) in values.iter_mut().enumerate() {
                        if *v == Val::Null {
                            continue;
                        }
                        if ctx.version && cols.names.get(c).is_some_and(|n| n == "version") {
                            // Item 10: the text after the version number names the platform.
                            if let Val::Str(text) = v
                                && let Some(at) = text.windows(4).position(|w| w == b" on ")
                            {
                                let mut tail = Val::Str(text.split_off(at));
                                s.mask(c, Rule::Item(10), &mut tail);
                            }
                            continue;
                        }
                        if server_version_value(cols, c, &row) {
                            // Item 12: the text after the version number names the server.
                            // The tail is empty on PostgreSQL, so the rule counts each value.
                            if let Val::Str(text) = v {
                                let at = text.iter().position(|&b| b == b' ').unwrap_or(text.len());
                                let mut tail = Val::Str(text.split_off(at));
                                s.mask(c, Rule::Item(12), &mut tail);
                            }
                            continue;
                        }
                        if let Some(item) = column_rule(ctx, cols, c, &row) {
                            s.mask(c, Rule::Item(item), v);
                        }
                    }
                }
            }
            "ErrorResponse" | "NoticeResponse" => {
                if let Val::List(fields) = &mut vals[0] {
                    let mut kept = Vec::new();
                    for pair in fields.chunks(2) {
                        if let [Val::Byte(code @ (b'F' | b'L' | b'R')), v] = pair {
                            let mut v = v.clone();
                            s.mask(usize::from(*code), Rule::Item(10), &mut v);
                        } else {
                            kept.extend_from_slice(pair);
                        }
                    }
                    *fields = kept;
                }
            }
            "ParameterStatus" => {
                if vals[0].bytes() == Some(b"server_version")
                    && let Val::Str(text) = &mut vals[1]
                {
                    let at = text.iter().position(|&b| b == b' ').unwrap_or(text.len());
                    let mut tail = Val::Str(text.split_off(at));
                    s.mask(1, Rule::Item(12), &mut tail);
                }
            }
            "BackendKeyData" => {
                s.mask(0, Rule::Item(13), &mut vals[0]);
                let len = vals[1].bytes().map_or(0, <[u8]>::len);
                s.mask(1, Rule::Rule(2), &mut vals[1]);
                vals[1] = Val::Int(len as i64);
            }
            "NotificationResponse" => s.mask(0, Rule::Item(13), &mut vals[0]),
            "AuthenticationSASLContinue" | "AuthenticationSASLFinal" => {
                // The shape stays: the attribute names and the iteration count.
                let text =
                    String::from_utf8_lossy(vals[0].bytes().unwrap_or_default()).into_owned();
                let shape: Vec<String> = text
                    .split(',')
                    .map(|a| match a.split_once('=') {
                        Some(("i", n)) => format!("i={n}"),
                        Some((k, _)) => format!("{k}=..."),
                        None => a.to_string(),
                    })
                    .collect();
                s.mask(0, Rule::Rule(3), &mut vals[0]);
                vals[0] = Val::Str(shape.join(",").into_bytes());
            }
            _ => {}
        }
        s.msg.vals = vals;
        out.push(s);
    }
    join_copy(&mut out);
    if !ctx.ordered {
        sort_rows(&mut out);
    }
    if ctx.explain {
        // Item 1: the number of plan lines is part of the plan shape.
        let mut first = true;
        out.retain(|s| {
            let keep = s.msg.name() != "DataRow" || first;
            if s.msg.name() == "DataRow" {
                first = false;
            }
            keep
        });
    }
    out
}

/// Joins each run of `CopyData` into one message (section 5.2).
fn join_copy(out: &mut Vec<Shown>) {
    let mut joined: Vec<Shown> = Vec::with_capacity(out.len());
    for s in out.drain(..) {
        if s.msg.name() == "CopyData"
            && let Some(last) = joined.last_mut()
            && last.msg.name() == "CopyData"
        {
            if let (Val::Str(a), Some(b)) = (&mut last.msg.vals[0], s.msg.vals[0].bytes()) {
                a.extend_from_slice(b);
            }
            last.masks.push(Mask { slot: 0, rule: Rule::Rule(1), orig: Val::Null });
            continue;
        }
        joined.push(s);
    }
    *out = joined;
}

/// Sorts each run of `DataRow`, so the rows compare as a multiset.
fn sort_rows(out: &mut [Shown]) {
    let mut i = 0;
    while i < out.len() {
        let start = i;
        while i < out.len() && out[i].msg.name() == "DataRow" {
            i += 1;
        }
        if i - start > 1 {
            out[start..i].sort_by(|a, b| a.msg.vals.cmp(&b.msg.vals));
            out[start].masks.push(Mask { slot: usize::MAX, rule: Rule::Rule(0), orig: Val::Null });
        }
        i = i.max(start + 1);
    }
}

fn count(outcome: &mut Outcome, rule: Rule, differed: bool) {
    let c = match rule {
        Rule::Item(n) => &mut outcome.items[n - 1],
        Rule::Rule(n) => &mut outcome.rules[n],
    };
    c.applied += 1;
    c.differed += usize::from(differed);
}

fn startup_has_replication(gs: &[Group<'_>]) -> bool {
    gs.first().is_some_and(|g| {
        g.front.iter().any(|m| {
            m.name() == "StartupMessage"
                && m.vals
                    .get(1)
                    .map(Val::list)
                    .unwrap_or_default()
                    .chunks(2)
                    .any(|p| p[0].bytes() == Some(b"replication"))
        })
    })
}

/// Compares the backend messages of `got` with those of `want`. The groups in `skip` are left out.
pub(crate) fn compare(want: &[Line], got: &[Line], skip: &BTreeSet<Place>) -> Outcome {
    let mut out = Outcome::default();
    let (want, got) = (groups(want), groups(got));
    let empty = Vec::new();
    for (&conn, wgs) in &want {
        let ggs = got.get(&conn).unwrap_or(&empty);
        let replication = startup_has_replication(wgs);
        let mut prepared = BTreeMap::new();
        let (mut wcols, mut gcols) = (Columns::default(), Columns::default());
        for i in 0..wgs.len().max(ggs.len()) {
            let blank = Group::default();
            let wg = wgs.get(i).unwrap_or(&blank);
            let gg = ggs.get(i).unwrap_or(&blank);
            let ctx =
                context(if wg.front.is_empty() { gg } else { wg }, &mut prepared, replication);
            if skip.contains(&(conn, i)) {
                out.unstable += 1;
                continue;
            }
            out.groups += 1;
            out.messages += wg.back.len();
            let w = show(wg, &ctx, &mut wcols);
            let g = show(gg, &ctx, &mut gcols);
            for (k, ws) in w.iter().enumerate() {
                let gs = g.get(k);
                for m in &ws.masks {
                    let other = gs.and_then(|gs| {
                        gs.masks.iter().find(|o| o.slot == m.slot && o.rule == m.rule)
                    });
                    count(&mut out, m.rule, other.is_none_or(|o| o.orig != m.orig));
                }
                if gs.is_none_or(|gs| gs.msg != ws.msg) && !out.diffs.contains_key(&(conn, i)) {
                    let got = gs.map_or("nothing".to_string(), |gs| format_msg(&gs.msg));
                    out.diffs
                        .insert((conn, i), format!("want {}\n    got  {got}", format_msg(&ws.msg)));
                }
            }
            if g.len() > w.len() && !out.diffs.contains_key(&(conn, i)) {
                out.diffs.insert(
                    (conn, i),
                    format!("want nothing\n    got  {}", format_msg(&g[w.len()].msg)),
                );
            }
        }
    }
    for (&conn, ggs) in &got {
        if !want.contains_key(&conn) && !ggs.is_empty() {
            out.diffs.insert((conn, 0), "want no connection\n    got  a connection".into());
        }
    }
    out
}

/// The unstable filter of spec/21 section 21.1: the groups whose answers differ between two runs on the oracle.
pub(crate) fn unstable(first: &[Line], second: &[Line]) -> BTreeSet<Place> {
    compare(first, second, &BTreeSet::new()).diffs.into_keys().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::parse_line;

    fn lines(text: &str) -> Vec<Line> {
        text.lines().map(|l| parse_line(l).unwrap()).collect()
    }

    fn run(want: &str, got: &str) -> Outcome {
        compare(&lines(want), &lines(got), &BTreeSet::new())
    }

    const START: &str = "1 F StartupMessage 3.0 \"user\" \"postgres\"\n1 B AuthenticationSASL \"SCRAM-SHA-256\"\n1 F SASLInitialResponse \"SCRAM-SHA-256\" \"n,,n=,r=abc\"\n";

    #[test]
    fn equal_traces_have_no_difference() {
        let t = format!(
            "{START}1 B AuthenticationSASLContinue \"r=abcdef,s=c2FsdA==,i=4096\"\n1 B BackendKeyData 77 \"key1\"\n1 B ReadyForQuery I\n1 F Query \"SELECT 1\"\n1 B DataRow [\"1\"]\n1 B ReadyForQuery I"
        );
        let o = run(&t, &t);
        assert!(o.diffs.is_empty(), "{:?}", o.diffs);
        assert_eq!(o.groups, 2);
        assert_eq!(o.items[12], Count { applied: 1, differed: 0 });
    }

    #[test]
    fn the_process_id_the_key_and_the_nonce_are_out() {
        let a = format!(
            "{START}1 B AuthenticationSASLContinue \"r=abcdef,s=c2FsdA==,i=4096\"\n1 B BackendKeyData 77 \"key1\"\n1 B ParameterStatus \"server_version\" \"19beta4 (rupg)\"\n1 B ReadyForQuery I"
        );
        let b = format!(
            "{START}1 B AuthenticationSASLContinue \"r=abcxyz,s=b3RoZXI=,i=4096\"\n1 B BackendKeyData 99 \"key2\"\n1 B ParameterStatus \"server_version\" \"19beta4\"\n1 B ReadyForQuery I"
        );
        let o = run(&a, &b);
        assert!(o.diffs.is_empty(), "{:?}", o.diffs);
        assert_eq!(o.items[12], Count { applied: 1, differed: 1 });
        assert_eq!(o.items[11], Count { applied: 1, differed: 1 });
        assert_eq!(o.rules[2], Count { applied: 1, differed: 1 });
        assert_eq!(o.rules[3], Count { applied: 1, differed: 1 });
        let c = a.replace("\"key1\"", "\"longer key\"");
        assert_eq!(run(&a, &c).diffs.len(), 1);
    }

    #[test]
    fn rows_without_order_by_are_a_multiset() {
        let q = |sql: &str, rows: &str| format!("1 F Query \"{sql}\"\n{rows}1 B ReadyForQuery I");
        let a_then_b = "1 B DataRow [\"a\"]\n1 B DataRow [\"b\"]\n";
        let b_then_a = "1 B DataRow [\"b\"]\n1 B DataRow [\"a\"]\n";
        assert!(
            run(&q("SELECT x FROM t", a_then_b), &q("SELECT x FROM t", b_then_a)).diffs.is_empty()
        );
        assert_eq!(
            run(
                &q("SELECT x FROM t ORDER BY x", a_then_b),
                &q("SELECT x FROM t ORDER BY x", b_then_a)
            )
            .diffs
            .len(),
            1
        );
        assert_eq!(
            run(&q("SELECT x FROM t", a_then_b), &q("SELECT x FROM t", "1 B DataRow [\"a\"]\n"))
                .diffs
                .len(),
            1
        );
    }

    #[test]
    fn error_text_counts_and_the_source_fields_do_not() {
        let e = |m: &str, line: &str| {
            format!(
                "1 F Query \"SELECT 1 +\"\n1 B ErrorResponse S \"ERROR\" C \"42601\" M \"{m}\" F \"scan.l\" L \"{line}\" R \"scanner_yyerror\"\n1 B ReadyForQuery I"
            )
        };
        let o = run(&e("syntax error", "1"), &e("syntax error", "2"));
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[9], Count { applied: 3, differed: 1 });
        assert_eq!(run(&e("syntax error", "1"), &e("syntax error!", "1")).diffs.len(), 1);
    }

    #[test]
    fn time_statistics_and_paths_are_out() {
        let q = |sql: &str, desc: &str, row: &str| {
            format!(
                "1 F Query \"{sql}\"\n1 B RowDescription [{desc}]\n1 B DataRow [{row}]\n1 B ReadyForQuery I"
            )
        };
        let col = |name: &str, ty: i64| format!("(\"{name}\" 0 0 {ty} 8 -1 0)");
        let now = col("now", 1184);
        let o = run(
            &q("SELECT now()", &now, "\"2026-10-07 01:00:00+00\""),
            &q("SELECT now()", &now, "\"2026-10-07 01:00:01+00\""),
        );
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[8], Count { applied: 1, differed: 1 });
        let t = col("t", 1184);
        let o = run(&q("SELECT t FROM x", &t, "\"1\""), &q("SELECT t FROM x", &t, "\"2\""));
        assert_eq!(o.diffs.len(), 1);
        let d = format!("{} {}", col("name", 25), col("setting", 25));
        let o = run(
            &q(
                "SELECT name, setting FROM pg_settings WHERE name = 'data_directory'",
                &d,
                "\"data_directory\" \"/a\"",
            ),
            &q(
                "SELECT name, setting FROM pg_settings WHERE name = 'data_directory'",
                &d,
                "\"data_directory\" \"/b\"",
            ),
        );
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[14].differed, 1);
        let s = col("seq_scan", 20);
        let o = run(
            &q("SELECT seq_scan FROM pg_stat_user_tables", &s, "\"1\""),
            &q("SELECT seq_scan FROM pg_stat_user_tables", &s, "\"5\""),
        );
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[2].differed, 1);
    }

    #[test]
    fn user_oids_and_wait_events_are_out() {
        let q = |sql: &str, desc: &str, row: &str| {
            format!(
                "1 F Query \"{sql}\"\n1 B RowDescription [{desc}]\n1 B DataRow [{row}]\n1 B ReadyForQuery I"
            )
        };
        let col = |name: &str, ty: i64| format!("(\"{name}\" 0 0 {ty} 4 -1 0)");
        let d = format!("{} {}", col("oid", 26), col("datname", 19));
        let sql = "SELECT oid, datname FROM pg_database";
        let o = run(&q(sql, &d, "\"16390\" \"a\""), &q(sql, &d, "\"16391\" \"a\""));
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[1], Count { applied: 1, differed: 1 });
        let o = run(&q(sql, &d, "\"5\" \"a\""), &q(sql, &d, "\"6\" \"a\""));
        assert_eq!(o.diffs.len(), 1);
        let d = format!("{} {}", col("backend_type", 25), col("wait_event", 25));
        let sql = "SELECT backend_type, wait_event FROM pg_stat_activity";
        let o = run(
            &q(sql, &d, "\"background writer\" \"BgwriterMain\""),
            &q(sql, &d, "\"background writer\" \"BgwriterHibernate\""),
        );
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[2], Count { applied: 1, differed: 1 });
    }

    #[test]
    fn the_suffix_of_server_version_in_rows_is_out() {
        let q = |sql: &str, desc: &str, row: &str| {
            format!(
                "1 F Query \"{sql}\"\n1 B RowDescription [{desc}]\n1 B DataRow [{row}]\n1 B ReadyForQuery I"
            )
        };
        let col = |name: &str| format!("(\"{name}\" 0 0 25 -1 -1 0)");
        let o = run(
            &q("SHOW server_version", &col("server_version"), "\"19.0\""),
            &q("SHOW server_version", &col("server_version"), "\"19.0 (rupg 0.1.0)\""),
        );
        assert!(o.diffs.is_empty());
        assert_eq!(o.items[11], Count { applied: 1, differed: 1 });
        let d = format!("{} {}", col("name"), col("setting"));
        let o = run(
            &q("SHOW ALL", &d, "\"server_version\" \"19.0\""),
            &q("SHOW ALL", &d, "\"server_version\" \"18.0\""),
        );
        assert_eq!(o.diffs.len(), 1);
    }

    #[test]
    fn language_c_is_found() {
        assert!(language_c("create function f() returns int as 'x' language c;"));
        assert!(language_c("create function f() returns int language 'c' as 'x'"));
        assert!(!language_c("create function f() returns int language sql as 'select 1'"));
        assert!(!language_c("create function f() language plpgsql"));
    }

    #[test]
    fn copy_data_is_joined() {
        let a = "1 F Query \"COPY t TO STDOUT\"\n1 B CopyOutResponse 0 []\n1 B CopyData \"a\\n\"\n1 B CopyData \"b\\n\"\n1 B CopyDone\n1 B ReadyForQuery I";
        let b = "1 F Query \"COPY t TO STDOUT\"\n1 B CopyOutResponse 0 []\n1 B CopyData \"a\\nb\\n\"\n1 B CopyDone\n1 B ReadyForQuery I";
        let o = run(a, b);
        assert!(o.diffs.is_empty(), "{:?}", o.diffs);
        assert_eq!(o.rules[1].applied, 1);
    }

    #[test]
    fn two_runs_find_the_unstable_groups() {
        let run = |r: &str| {
            format!(
                "1 F Query \"SELECT 1\"\n1 B DataRow [\"1\"]\n1 B ReadyForQuery I\n1 F Query \"SELECT random()\"\n1 B DataRow [\"{r}\"]\n1 B ReadyForQuery I"
            )
        };
        let (trace, first, second) = (run("0.1"), run("0.2"), run("0.3"));
        let skip = unstable(&lines(&first), &lines(&second));
        assert_eq!(skip, [(1, 1)].into());
        let o = compare(&lines(&trace), &lines(&first), &skip);
        assert!(o.diffs.is_empty());
        assert_eq!((o.groups, o.unstable), (1, 1));
        assert_eq!(unstable(&lines(&first), &lines(&first)), BTreeSet::new());
    }

    #[test]
    fn a_skipped_group_is_unstable() {
        let a = "1 F Query \"SELECT random()\"\n1 B DataRow [\"0.1\"]\n1 B ReadyForQuery I";
        let b = "1 F Query \"SELECT random()\"\n1 B DataRow [\"0.2\"]\n1 B ReadyForQuery I";
        let skip: BTreeSet<Place> = [(1, 0)].into();
        let o = compare(&lines(a), &lines(b), &skip);
        assert!(o.diffs.is_empty());
        assert_eq!(o.unstable, 1);
    }
}
