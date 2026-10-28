//! The generated queries of spec/21 section 21.4.5, with the differential oracle. The grammar is `gram.y` of the oracle source at the pin, with a weight on each production.
//!
//! A run starts from a seed. The seed makes a small schema with data, and then each statement. The statements start from the DML productions of the grammar, so that a statement cannot change the cluster or the host. The weights start at 1, and the weight of a production that is only a name is 32 times the weight. After each round, the weight of each production that no statement of the round used goes up to two times its value, to at most 64, and the weight of each production that a statement used goes back to 1. So the same seed always gives the same statements.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::client::{self, Conn, Target};
use crate::message::Msg;
use crate::slt::{self, Sort};

/// The productions that a statement starts from.
pub(crate) const STARTS: [&str; 4] = ["SelectStmt", "InsertStmt", "UpdateStmt", "DeleteStmt"];
/// The statements of one round of the weights.
pub(crate) const ROUND: usize = 200;
/// The highest weight of a production.
const MAX_WEIGHT: u32 = 64;
/// The depth after which each choice takes the production with the lowest height.
const MAX_DEPTH: usize = 9;
/// The tokens after which each choice takes the production with the lowest height.
const MAX_TOKENS: usize = 80;
/// The factor of the weight of a production that is only the token `IDENT`. A name rule such as `ColId` has a production for each keyword that can be a name, and without the factor most names are keywords.
const IDENT_FACTOR: u32 = 32;
/// The names that the token `IDENT` gives: the names of the schema, some functions and some types.
const NAMES: [&str; 18] = [
    "t0", "t1", "t2", "c0", "c1", "c2", "c3", "c4", "x", "abs", "lower", "length", "coalesce",
    "count", "sum", "int4", "text", "numeric",
];

/// One symbol of a production: a rule or a token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Sym {
    Rule(usize),
    Token(String),
}

/// One rule of the grammar, with its productions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Rule {
    pub(crate) name: String,
    pub(crate) alts: Vec<Vec<Sym>>,
}

/// The rules of `gram.y` and the text of each keyword token.
#[derive(Debug)]
pub(crate) struct Grammar {
    pub(crate) rules: Vec<Rule>,
    keywords: HashMap<String, String>,
    /// The height of each production: the depth of its smallest expansion, or `u32::MAX` when it has no expansion.
    heights: Vec<Vec<u32>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Word(String),
    Char(String),
    Colon,
    Bar,
    Semi,
}

/// The tokens of the rules section of a Bison file, without the actions, the comments and the `%prec` clauses.
fn lex(text: &str) -> Result<Vec<Tok>, String> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    // Skips a C string or a C character literal that starts at `i`, and returns the index after it.
    let quoted = |mut i: usize, q: u8| {
        i += 1;
        while i < b.len() && b[i] != q {
            i += if b[i] == b'\\' { 2 } else { 1 };
        }
        i + 1
    };
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if b[i..].starts_with(b"/*") {
            i = text[i + 2..].find("*/").map_or(b.len(), |e| i + 2 + e + 2);
        } else if b[i..].starts_with(b"//") {
            i = text[i..].find('\n').map_or(b.len(), |e| i + e);
        } else if c == b'{' {
            let mut depth = 0;
            while i < b.len() {
                match b[i] {
                    b'"' | b'\'' => {
                        i = quoted(i, b[i]);
                        continue;
                    }
                    _ if b[i..].starts_with(b"/*") => {
                        i = text[i + 2..].find("*/").map_or(b.len(), |e| i + 2 + e + 2);
                        continue;
                    }
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        } else if c == b'\'' {
            let end = quoted(i, b'\'');
            out.push(Tok::Char(text[i + 1..end - 1].to_string()));
            i = end;
        } else if c == b':' {
            out.push(Tok::Colon);
            i += 1;
        } else if c == b'|' {
            out.push(Tok::Bar);
            i += 1;
        } else if c == b';' {
            out.push(Tok::Semi);
            i += 1;
        } else if c == b'%' {
            let start = i + 1;
            i = start;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            match &text[start..i] {
                "prec" => {
                    // The token of the precedence is not a symbol of the production.
                    while i < b.len() && b[i].is_ascii_whitespace() {
                        i += 1;
                    }
                    if b.get(i) == Some(&b'\'') {
                        i = quoted(i, b'\'');
                    } else {
                        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                            i += 1;
                        }
                    }
                }
                "empty" => {}
                other => return Err(format!("unknown directive %{other} in the rules")),
            }
        } else if c.is_ascii_alphanumeric() || c == b'_' || c == b'.' {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            out.push(Tok::Word(text[start..i].to_string()));
        } else {
            return Err(format!("unexpected character {:?} in the rules", c as char));
        }
    }
    Ok(out)
}

impl Grammar {
    /// Reads `src/backend/parser/gram.y` and `src/include/parser/kwlist.h` of a PostgreSQL source tree.
    pub(crate) fn load(src: &Path) -> Result<Grammar, String> {
        let read = |p: &str| {
            let path = src.join(p);
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
        };
        Grammar::parse(&read("src/backend/parser/gram.y")?, &read("src/include/parser/kwlist.h")?)
    }

    pub(crate) fn parse(gram: &str, kwlist: &str) -> Result<Grammar, String> {
        let mut parts = gram.split("\n%%");
        let rules_text = parts.nth(1).ok_or("gram.y has no rules section")?;
        let toks = lex(rules_text)?;
        let mut raw: Vec<(String, Vec<Vec<Tok>>)> = Vec::new();
        let mut i = 0;
        while i < toks.len() {
            let (Some(Tok::Word(name)), Some(Tok::Colon)) = (toks.get(i), toks.get(i + 1)) else {
                return Err(format!(
                    "rule {} does not start with a name and a colon",
                    raw.len() + 1
                ));
            };
            i += 2;
            let mut alts = vec![Vec::new()];
            loop {
                match toks.get(i) {
                    Some(Tok::Semi) => break,
                    Some(Tok::Bar) => alts.push(Vec::new()),
                    Some(t @ (Tok::Word(_) | Tok::Char(_))) => {
                        alts.last_mut().ok_or("no production")?.push(t.clone());
                    }
                    Some(Tok::Colon) | None => return Err(format!("the rule {name} has no end")),
                }
                i += 1;
            }
            i += 1;
            raw.push((name.clone(), alts));
        }
        let index: HashMap<&str, usize> =
            raw.iter().enumerate().map(|(n, (name, _))| (name.as_str(), n)).collect();
        let rules: Vec<Rule> = raw
            .iter()
            .map(|(name, alts)| Rule {
                name: name.clone(),
                alts: alts
                    .iter()
                    .map(|alt| {
                        alt.iter()
                            .map(|t| match t {
                                Tok::Word(w) => match index.get(w.as_str()) {
                                    Some(&r) => Sym::Rule(r),
                                    None => Sym::Token(w.clone()),
                                },
                                Tok::Char(c) => Sym::Token(format!("'{c}'")),
                                _ => Sym::Token(String::new()),
                            })
                            .collect()
                    })
                    .collect(),
            })
            .collect();
        let mut keywords = HashMap::new();
        for line in kwlist.lines().filter(|l| l.starts_with("PG_KEYWORD(\"")) {
            let mut fields = line["PG_KEYWORD(".len()..].split(',').map(str::trim);
            if let (Some(text), Some(token)) = (fields.next(), fields.next()) {
                keywords.insert(token.to_string(), text.trim_matches('"').to_uppercase());
            }
        }
        let mut g = Grammar { rules, keywords, heights: Vec::new() };
        g.heights = g.compute_heights();
        Ok(g)
    }

    /// The number of productions.
    pub(crate) fn productions(&self) -> usize {
        self.rules.iter().map(|r| r.alts.len()).sum()
    }

    /// The rule with the name.
    pub(crate) fn rule(&self, name: &str) -> Option<usize> {
        self.rules.iter().position(|r| r.name == name)
    }

    fn compute_heights(&self) -> Vec<Vec<u32>> {
        let mut rule_height = vec![u32::MAX; self.rules.len()];
        let mut heights: Vec<Vec<u32>> =
            self.rules.iter().map(|r| vec![u32::MAX; r.alts.len()]).collect();
        loop {
            let mut changed = false;
            for (r, rule) in self.rules.iter().enumerate() {
                for (a, alt) in rule.alts.iter().enumerate() {
                    let mut h = 0u32;
                    for sym in alt {
                        let s = match sym {
                            Sym::Rule(x) => rule_height[*x],
                            Sym::Token(t) if self.text_of(t).is_none() => u32::MAX,
                            Sym::Token(_) => 0,
                        };
                        h = h.max(s);
                    }
                    let h = h.saturating_add(1);
                    if h < heights[r][a] {
                        heights[r][a] = h;
                        changed = true;
                    }
                    rule_height[r] = rule_height[r].min(h);
                }
            }
            if !changed {
                return heights;
            }
        }
    }

    /// The fixed text of a token, `Some("")` for a token whose text the generator picks, or `None` for a token that the generator never writes.
    fn text_of(&self, token: &str) -> Option<String> {
        if let Some(c) = token.strip_prefix('\'').and_then(|t| t.strip_suffix('\'')) {
            return Some(c.to_string());
        }
        if let Some(k) = self.keywords.get(token) {
            return Some(k.clone());
        }
        let text = match token {
            "IDENT" | "UIDENT" | "FCONST" | "SCONST" | "USCONST" | "BCONST" | "XCONST" | "Op"
            | "ICONST" | "PARAM" => "",
            "TYPECAST" => "::",
            "DOT_DOT" => "..",
            "COLON_EQUALS" => ":=",
            "EQUALS_GREATER" => "=>",
            "LESS_EQUALS" => "<=",
            "GREATER_EQUALS" => ">=",
            "NOT_EQUALS" => "<>",
            "FORMAT_LA" => "FORMAT",
            "NOT_LA" => "NOT",
            "NULLS_LA" => "NULLS",
            "WITH_LA" => "WITH",
            "WITHOUT_LA" => "WITHOUT",
            _ => return None,
        };
        Some(text.to_string())
    }

    /// The productions that the rules of `starts` reach, as pairs of a rule and a production.
    pub(crate) fn reachable(&self, starts: &[usize]) -> Vec<(usize, usize)> {
        let mut seen = vec![false; self.rules.len()];
        let mut stack = starts.to_vec();
        let mut out = Vec::new();
        while let Some(r) = stack.pop() {
            if std::mem::replace(&mut seen[r], true) {
                continue;
            }
            for (a, alt) in self.rules[r].alts.iter().enumerate() {
                if self.heights[r][a] == u32::MAX {
                    continue;
                }
                out.push((r, a));
                stack.extend(alt.iter().filter_map(|s| match s {
                    Sym::Rule(x) => Some(*x),
                    Sym::Token(_) => None,
                }));
            }
        }
        out.sort_unstable();
        out
    }
}

/// SplitMix64.
#[derive(Clone, Debug)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number below `n`, which must not be 0.
    pub(crate) fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).unwrap_or_default()
    }

    fn pick<'a>(&mut self, list: &[&'a str]) -> &'a str {
        list[self.below(list.len())]
    }
}

/// The weights of the productions, and the productions that the statements of the current round used.
#[derive(Clone, Debug)]
pub(crate) struct Weights {
    base: Vec<Vec<u32>>,
    weights: Vec<Vec<u32>>,
    used: Vec<Vec<bool>>,
}

impl Weights {
    pub(crate) fn new(g: &Grammar) -> Weights {
        let base = |alt: &Vec<Sym>| match alt.as_slice() {
            [Sym::Token(t)] if t == "IDENT" => IDENT_FACTOR,
            _ => 1,
        };
        Weights {
            base: g.rules.iter().map(|r| r.alts.iter().map(base).collect()).collect(),
            weights: g.rules.iter().map(|r| vec![1; r.alts.len()]).collect(),
            used: g.rules.iter().map(|r| vec![false; r.alts.len()]).collect(),
        }
    }

    /// Ends a round: a production that no statement used gets two times its weight, to at most 64, and a production that a statement used gets 1.
    pub(crate) fn end_round(&mut self) {
        for (ws, us) in self.weights.iter_mut().zip(&mut self.used) {
            for (w, u) in ws.iter_mut().zip(us.iter_mut()) {
                *w = if *u { 1 } else { (*w * 2).min(MAX_WEIGHT) };
                *u = false;
            }
        }
    }
}

/// Writes one statement that starts from the rule `start`. It marks the productions that it uses in `weights`, and also in `ever` for the count of the run.
pub(crate) fn statement(
    g: &Grammar,
    start: usize,
    rng: &mut Rng,
    weights: &mut Weights,
    ever: &mut [Vec<bool>],
) -> String {
    let mut out: Vec<String> = Vec::new();
    // Each entry is a symbol and its depth. The symbols of a production go on the stack in reverse order.
    let mut stack = vec![(Sym::Rule(start), 0usize)];
    while let Some((sym, depth)) = stack.pop() {
        let r = match sym {
            Sym::Rule(r) => r,
            Sym::Token(t) => {
                out.push(token(g, &t, rng));
                continue;
            }
        };
        let heights = &g.heights[r];
        let a = if depth >= MAX_DEPTH || out.len() >= MAX_TOKENS {
            let low = heights.iter().copied().min().unwrap_or(u32::MAX);
            let lows: Vec<usize> = (0..heights.len()).filter(|&a| heights[a] == low).collect();
            lows[rng.below(lows.len())]
        } else {
            let ok: Vec<usize> = (0..heights.len()).filter(|&a| heights[a] != u32::MAX).collect();
            let weight = |a: usize| u64::from(weights.weights[r][a] * weights.base[r][a]);
            let total: u64 = ok.iter().map(|&a| weight(a)).sum();
            let mut at = rng.next() % total.max(1);
            let mut chosen = ok[0];
            for &a in &ok {
                let w = weight(a);
                if at < w {
                    chosen = a;
                    break;
                }
                at -= w;
            }
            chosen
        };
        weights.used[r][a] = true;
        ever[r][a] = true;
        for s in g.rules[r].alts[a].iter().rev() {
            stack.push((s.clone(), depth + 1));
        }
    }
    out.retain(|t| !t.is_empty());
    out.join(" ")
}

/// The text of one token.
fn token(g: &Grammar, t: &str, rng: &mut Rng) -> String {
    match t {
        "IDENT" => rng.pick(&NAMES).to_string(),
        "UIDENT" => format!("U&\"{}\"", rng.pick(&NAMES)),
        "ICONST" => rng.pick(&["0", "1", "2", "3", "10", "2147483647"]).to_string(),
        "FCONST" => rng.pick(&["0.5", "1.0", "2.5e3", "1e-3"]).to_string(),
        "SCONST" => {
            rng.pick(&["'a'", "''", "'1'", "'abc'", "'t'", "'2020-01-01'", "'{1,2}'"]).to_string()
        }
        "USCONST" => "U&'a'".to_string(),
        "BCONST" => "B'101'".to_string(),
        "XCONST" => "X'1F'".to_string(),
        "Op" => rng.pick(&["||", "~~", "!~~", "@>", "<@", "&&", "<<", ">>", "~", "!="]).to_string(),
        "PARAM" => "$1".to_string(),
        _ => g.text_of(t).unwrap_or_default(),
    }
}

/// The schema and the data of a seed: three tables of two to four columns, and up to eight rows in each.
pub(crate) fn schema(rng: &mut Rng) -> Vec<String> {
    let types = ["int4", "int8", "text", "bool", "numeric", "float8"];
    let mut sql = Vec::new();
    let mut column = 0;
    for t in 0..3 {
        let n = 2 + rng.below(3);
        let mut cols: Vec<(String, &str)> = Vec::new();
        for _ in 0..n {
            // A column name can be in more than one table, so that a name can be ambiguous.
            column = (column + 1 + rng.below(2)) % 5;
            let ty = types[rng.below(types.len())];
            if !cols.iter().any(|(c, _)| *c == format!("c{column}")) {
                cols.push((format!("c{column}"), ty));
            }
        }
        let defs: Vec<String> = cols.iter().map(|(c, ty)| format!("{c} {ty}")).collect();
        sql.push(format!("CREATE TABLE t{t} ({})", defs.join(", ")));
        for _ in 0..rng.below(9) {
            let vals: Vec<String> = cols.iter().map(|(_, ty)| value(ty, rng)).collect();
            sql.push(format!("INSERT INTO t{t} VALUES ({})", vals.join(", ")));
        }
    }
    sql
}

fn value(ty: &str, rng: &mut Rng) -> String {
    if rng.below(6) == 0 {
        return "NULL".into();
    }
    match ty {
        "int4" | "int8" => rng.pick(&["0", "1", "-1", "2", "7", "100"]).to_string(),
        "text" => rng.pick(&["'a'", "''", "'b'", "'abc'", "'A'"]).to_string(),
        "bool" => rng.pick(&["true", "false"]).to_string(),
        _ => rng.pick(&["0", "1.5", "-2.25", "1e3", "0.1"]).to_string(),
    }
}

/// The answer of a generated statement: its error, or its command tag and its rows in text order.
pub(crate) fn answer(msgs: &[Msg]) -> String {
    match slt::answer(None, msgs) {
        error if error.starts_with("error ") => error,
        tag => format!("{tag}\n{}", slt::answer(Some(Sort::Row), msgs)),
    }
}

/// Runs a statement in a transaction that rolls back, so that each statement starts from the data of the seed.
fn run_one(c: &mut Conn, sql: &str) -> Result<String, String> {
    c.execute("BEGIN")?;
    let msgs = c.query(sql)?;
    c.execute("ROLLBACK")?;
    Ok(answer(&msgs))
}

/// The servers of a run. The oracle has two databases, so that a statement whose answer is not the same on both is unstable.
pub(crate) struct Sides {
    pub(crate) oracle: Target,
    pub(crate) server: Target,
    pub(crate) timeout: Duration,
}

/// A statement whose answer on the server under test is not the answer of the oracle.
#[derive(Clone, Debug)]
pub(crate) struct Diff {
    pub(crate) number: usize,
    pub(crate) sql: String,
    pub(crate) oracle: String,
    pub(crate) server: String,
}

/// The result of a run.
#[derive(Clone, Debug, Default)]
pub(crate) struct Outcome {
    pub(crate) schema: Vec<String>,
    pub(crate) statements: usize,
    pub(crate) equal: usize,
    /// The stable statements that give an error on the oracle.
    pub(crate) errors: usize,
    pub(crate) unstable: Vec<usize>,
    pub(crate) diffs: Vec<Diff>,
    /// The productions that the run used, of the productions that the starts reach.
    pub(crate) used: usize,
    pub(crate) reachable: usize,
}

fn open(target: &Target, db: &str, timeout: Duration, reset: bool) -> Result<Conn, String> {
    if reset {
        client::reset_database(target, db)?;
    }
    let mut c = Conn::connect(target, db)?;
    c.set_timeout(timeout)?;
    // The oracle stops a statement after 5 s. A server that does not have the parameter gives an error here, and the read timeout stops the statement.
    c.query("SET statement_timeout = '5s'")?;
    Ok(c)
}

/// Runs `count` statements of the seed.
pub(crate) fn run(g: &Grammar, sides: &Sides, seed: u64, count: usize) -> Result<Outcome, String> {
    let mut rng = Rng::new(seed);
    let starts: Vec<usize> = STARTS
        .iter()
        .map(|s| g.rule(s).ok_or(format!("gram.y has no rule {s}")))
        .collect::<Result<_, _>>()?;
    let mut out = Outcome { schema: schema(&mut rng), ..Default::default() };
    let same = sides.oracle.addr == sides.server.addr;
    let server_db = if same { "gen_server" } else { "gen" };
    let mut a = open(&sides.oracle, "gen_a", Duration::from_secs(60), true)?;
    let mut b = open(&sides.oracle, "gen_b", Duration::from_secs(60), true)?;
    let mut s = open(&sides.server, server_db, sides.timeout, true);
    for sql in &out.schema {
        a.execute(sql)?;
        b.execute(sql)?;
        if let Ok(c) = &mut s {
            c.execute(sql)?;
        }
    }
    let mut weights = Weights::new(g);
    let mut ever: Vec<Vec<bool>> = g.rules.iter().map(|r| vec![false; r.alts.len()]).collect();
    for n in 1..=count {
        let start = starts[rng.below(starts.len())];
        let sql = statement(g, start, &mut rng, &mut weights, &mut ever);
        if n % ROUND == 0 {
            weights.end_round();
        }
        let want = run_one(&mut a, &sql)?;
        let again = run_one(&mut b, &sql)?;
        let got = match &mut s {
            Ok(c) => run_one(c, &sql),
            Err(e) => Err(e.clone()),
        };
        let got = match got {
            Ok(text) => text,
            Err(e) => {
                // The connection is lost. The next statement uses a new connection to the same database.
                s = open(&sides.server, server_db, sides.timeout, false);
                format!("no answer: {e}")
            }
        };
        out.statements += 1;
        if want != again || want.contains(" 57014 ") {
            out.unstable.push(n);
        } else if want == got {
            out.equal += 1;
            out.errors += usize::from(want.starts_with("error "));
        } else {
            out.diffs.push(Diff { number: n, sql, oracle: want, server: got });
        }
    }
    let reachable = g.reachable(&starts);
    out.used = reachable.iter().filter(|(r, a)| ever[*r][*a]).count();
    out.reachable = reachable.len();
    drop((a, b, s));
    let mut c = Conn::connect(&sides.oracle, "template1")?;
    for db in ["gen_a", "gen_b"] {
        c.execute(&format!("DROP DATABASE IF EXISTS {db} WITH (FORCE)"))?;
    }
    if let Ok(mut c) = Conn::connect(&sides.server, "template1") {
        let _ = c.execute(&format!("DROP DATABASE IF EXISTS {server_db} WITH (FORCE)"));
    }
    Ok(out)
}

/// The SQL file of the differences of a run: the schema, then each statement with the two answers as comments.
pub(crate) fn diff_file(seed: u64, out: &Outcome) -> String {
    let mut text = format!("-- rupg-compat generate --seed {seed}\n");
    for sql in &out.schema {
        text.push_str(&format!("{sql};\n"));
    }
    for d in &out.diffs {
        let comment = |s: &str| s.replace('\n', "\n--   ");
        text.push_str(&format!(
            "\n-- statement {}\n-- oracle: {}\n-- server: {}\n{};\n",
            d.number,
            comment(&d.oracle),
            comment(&d.server),
            d.sql
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAM: &str = "%token <str> IDENT ICONST\n%%\n\
        stmt: SelectStmt | /* EMPTY */ { $$ = NULL; } ;\n\
        SelectStmt: SELECT target_list from_clause\n\
            { SelectStmt *n = makeNode(SelectStmt); n->x = \"}\"; $$ = (Node *) n; }\n\
            ;\n\
        target_list: target_list ',' a_expr | a_expr ;\n\
        a_expr: ICONST | IDENT | a_expr '+' a_expr %prec '+' | '(' a_expr ')' | MODE_TYPE_NAME a_expr ;\n\
        from_clause: FROM IDENT | %empty ;\n\
        %%\nint x;\n";
    const KW: &str = "PG_KEYWORD(\"select\", SELECT, RESERVED_KEYWORD, AS_LABEL)\nPG_KEYWORD(\"from\", FROM, RESERVED_KEYWORD, AS_LABEL)\n";

    #[test]
    fn the_rules_parse_without_actions() {
        let g = Grammar::parse(GRAM, KW).unwrap();
        let names: Vec<&str> = g.rules.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["stmt", "SelectStmt", "target_list", "a_expr", "from_clause"]);
        assert_eq!(g.productions(), 12);
        assert_eq!(g.rules[0].alts[1], Vec::<Sym>::new());
        assert_eq!(g.rules[1].alts.len(), 1);
        assert_eq!(g.rules[3].alts[2], vec![Sym::Rule(3), Sym::Token("'+'".into()), Sym::Rule(3)]);
    }

    #[test]
    fn a_production_with_a_mode_token_has_no_height() {
        let g = Grammar::parse(GRAM, KW).unwrap();
        assert_eq!(g.heights[3][4], u32::MAX);
        assert_eq!(g.heights[3][0], 1);
        let reach = g.reachable(&[1]);
        assert!(!reach.contains(&(3, 4)));
        assert_eq!(reach.len(), 9);
    }

    #[test]
    fn the_same_seed_gives_the_same_statements() {
        let g = Grammar::parse(GRAM, KW).unwrap();
        let make = |seed| {
            let mut rng = Rng::new(seed);
            let mut w = Weights::new(&g);
            let mut ever: Vec<Vec<bool>> =
                g.rules.iter().map(|r| vec![false; r.alts.len()]).collect();
            (0..20).map(|_| statement(&g, 1, &mut rng, &mut w, &mut ever)).collect::<Vec<_>>()
        };
        assert_eq!(make(7), make(7));
        assert_ne!(make(7), make(8));
        for s in make(7) {
            assert!(s.starts_with("SELECT "), "{s}");
            assert!(!s.contains("MODE_TYPE_NAME"), "{s}");
        }
    }

    #[test]
    fn an_unused_production_gets_a_higher_weight() {
        let g = Grammar::parse(GRAM, KW).unwrap();
        let mut w = Weights::new(&g);
        w.used[3][0] = true;
        w.end_round();
        assert_eq!(w.weights[3][0], 1);
        assert_eq!(w.weights[3][1], 2);
        for _ in 0..10 {
            w.end_round();
        }
        assert_eq!(w.weights[3][1], MAX_WEIGHT);
    }

    #[test]
    fn the_schema_has_three_tables() {
        let sql = schema(&mut Rng::new(1));
        assert_eq!(sql.iter().filter(|s| s.starts_with("CREATE TABLE")).count(), 3);
    }
}
