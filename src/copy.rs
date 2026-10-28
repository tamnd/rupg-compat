//! The `COPY` suite of spec/05 section 5.13: `COPY TO` and `COPY FROM` in each format of the pin (text, csv, binary and json), with the options of each format and the errors, compared with the oracle message by message.
//!
//! The cases use one database on each side. The table `src` has a column of each of the common types and rows with special values. A `COPY TO` case compares the messages of the copy. A `COPY FROM` case empties the target table, sends the data, and compares the messages of the copy and then the rows of the table. The data of a round trip case is the output of the oracle for a `COPY TO`, so that the server must read what the oracle writes.

use crate::client::{self, Conn, Target};
use crate::message::{Msg, Val};

/// The database of the suite on each side.
pub(crate) const DATABASE: &str = "copy";

/// The schema and the data of the suite.
const SETUP: &[&str] = &[
    "CREATE TABLE src (id int4 PRIMARY KEY, i8 int8, n numeric, f float8, b bool, t text, d date, ts timestamp, tz timestamptz, iv interval, by bytea, j json, jb jsonb, a int4[], u uuid)",
    "INSERT INTO src VALUES (1, 9223372036854775807, 3.14159, 1.5, true, 'plain', '2024-02-29', '2024-02-29 12:34:56.789', '2024-02-29 12:34:56+02', '1 day 2 hours', '\\x00ff10', '{\"a\": [1, 2]}', '{\"b\": null, \"a\": 1}', '{1,NULL,3}', 'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11')",
    "INSERT INTO src VALUES (2, -1, -0.001, 'NaN', false, E'tab\\there', '1999-12-31', '1999-12-31 23:59:59', '1999-12-31 23:59:59-08', '-3 mons 4 days', '\\x', '[]', '[1, \"x\"]', '{}', '00000000-0000-0000-0000-000000000000')",
    "INSERT INTO src (id) VALUES (3)",
    "INSERT INTO src VALUES (4, 0, 1e20, '-Infinity', NULL, E'line\\nbreak, \"quoted\" \\\\ back', 'infinity', '-infinity', 'infinity', '0', '\\x5c0a', 'null', '\"s\"', '{{1,2},{3,4}}', NULL)",
    "INSERT INTO src VALUES (5, 42, 0.5, 1e-300, true, 'unicode é ü 日本', '2000-01-01', '2000-01-01 00:00:00', '2000-01-01 00:00:00+00', '1 year', '\\xdeadbeef', '{\"k\": \"v é\"}', '{\"k\": [true, false]}', '{-1}', 'ffffffff-ffff-ffff-ffff-ffffffffffff')",
    "INSERT INTO src VALUES (6, 7, 1, 0, false, '\\N', '2024-01-01', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)",
    "INSERT INTO src VALUES (7, 8, 2, -0.0, true, 'NULL', '2024-01-02', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)",
    "INSERT INTO src VALUES (8, 9, 3, 2.5, true, '', '2024-01-03', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)",
    "INSERT INTO src VALUES (9, 10, 4, 3.5, false, ',;|', '2024-01-04', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)",
    "CREATE TABLE dst (LIKE src INCLUDING ALL)",
    "CREATE TABLE small (a int4, b text, c int4 DEFAULT 7)",
];

/// The settings of each session, so that the output of the dates and the times does not depend on the server.
const SESSION: &[&str] =
    &["SET TimeZone = 'UTC'", "SET DateStyle = 'ISO, MDY'", "SET IntervalStyle = 'postgres'"];

/// The `COPY TO` cases: a name and the statement.
const OUT: &[(&str, &str)] = &[
    ("to text", "COPY src TO STDOUT"),
    ("to text delimiter null", "COPY src TO STDOUT (FORMAT text, DELIMITER '|', NULL 'NUL')"),
    ("to text header", "COPY src TO STDOUT (FORMAT text, HEADER)"),
    ("to text columns", "COPY src (id, t, j) TO STDOUT"),
    ("to text old syntax", "COPY src TO STDOUT WITH DELIMITER ',' NULL ''"),
    ("to csv", "COPY src TO STDOUT (FORMAT csv)"),
    ("to csv header", "COPY src TO STDOUT (FORMAT csv, HEADER, DELIMITER ';', NULL 'NULL')"),
    ("to csv quote escape", "COPY src TO STDOUT (FORMAT csv, QUOTE '''', ESCAPE '\\')"),
    ("to csv force quote all", "COPY src TO STDOUT (FORMAT csv, FORCE_QUOTE *)"),
    ("to csv force quote columns", "COPY src TO STDOUT (FORMAT csv, FORCE_QUOTE (t, n))"),
    ("to csv old syntax", "COPY src TO STDOUT WITH CSV HEADER"),
    ("to binary", "COPY src TO STDOUT (FORMAT binary)"),
    ("to json", "COPY src TO STDOUT (FORMAT json)"),
    ("to json force array", "COPY src TO STDOUT (FORMAT json, FORCE_ARRAY)"),
    ("to json columns", "COPY src (id, t) TO STDOUT (FORMAT json)"),
    (
        "to query",
        "COPY (SELECT id, t FROM src WHERE id > 2 ORDER BY id DESC) TO STDOUT (FORMAT csv)",
    ),
    ("to query values", "COPY (VALUES (1, 'a'), (NULL, NULL)) TO STDOUT"),
    ("to latin1", "COPY src TO STDOUT (ENCODING 'LATIN1')"),
    (
        "to latin1 ascii rows",
        "COPY (SELECT id, t FROM src WHERE id <> 5) TO STDOUT (ENCODING 'LATIN1')",
    ),
    ("to error binary delimiter", "COPY src TO STDOUT (FORMAT binary, DELIMITER ',')"),
    ("to error json header", "COPY src TO STDOUT (FORMAT json, HEADER)"),
    ("to error csv quote delimiter", "COPY src TO STDOUT (FORMAT csv, DELIMITER ',', QUOTE ',')"),
    ("to error format", "COPY src TO STDOUT (FORMAT xml)"),
    ("to error text force quote", "COPY src TO STDOUT (FORMAT text, FORCE_QUOTE *)"),
    ("to error force not null", "COPY src TO STDOUT (FORMAT csv, FORCE_NOT_NULL (t))"),
    ("to error force array csv", "COPY src TO STDOUT (FORMAT csv, FORCE_ARRAY)"),
    ("to error update", "COPY (UPDATE small SET a = 1) TO STDOUT"),
    ("to error column", "COPY src (nope) TO STDOUT"),
    ("to error table", "COPY nope TO STDOUT"),
    ("to error duplicate option", "COPY src TO STDOUT (FORMAT csv, FORMAT text)"),
];

/// The round trip cases: a name, the `COPY TO` on the oracle, and the `COPY FROM` of its output into `dst`.
const ROUND_TRIP: &[(&str, &str, &str)] = &[
    ("round trip text", "COPY src TO STDOUT", "COPY dst FROM STDIN"),
    (
        "round trip text delimiter null",
        "COPY src TO STDOUT (DELIMITER '|', NULL 'NUL')",
        "COPY dst FROM STDIN (DELIMITER '|', NULL 'NUL')",
    ),
    ("round trip text header", "COPY src TO STDOUT (HEADER)", "COPY dst FROM STDIN (HEADER MATCH)"),
    ("round trip csv", "COPY src TO STDOUT (FORMAT csv)", "COPY dst FROM STDIN (FORMAT csv)"),
    (
        "round trip csv header",
        "COPY src TO STDOUT (FORMAT csv, HEADER, FORCE_QUOTE *)",
        "COPY dst FROM STDIN (FORMAT csv, HEADER MATCH)",
    ),
    (
        "round trip binary",
        "COPY src TO STDOUT (FORMAT binary)",
        "COPY dst FROM STDIN (FORMAT binary)",
    ),
    (
        "round trip json into csv",
        "COPY src TO STDOUT (FORMAT json)",
        "COPY dst FROM STDIN (FORMAT csv)",
    ),
];

/// The `COPY FROM` cases with fixed data: a name, the statement and the data.
const IN: &[(&str, &str, &str)] = &[
    ("from text", "COPY small FROM STDIN", "1\ta\t2\n2\t\\N\t\\N\n\\N\tb\t3\n"),
    ("from text escapes", "COPY small FROM STDIN", "1\t\\x41\\101\\t\\\\\\n\t4\n"),
    ("from text end marker", "COPY small FROM STDIN", "1\ta\t1\n\\.\n"),
    ("from text columns", "COPY small (b, a) FROM STDIN", "x\t5\n"),
    ("from text where", "COPY small FROM STDIN WHERE a > 1", "1\ta\t1\n2\tb\t2\n3\tc\t3\n"),
    ("from text default", "COPY small FROM STDIN (DEFAULT 'D')", "1\ta\tD\n2\tD\t3\n"),
    ("from csv", "COPY small FROM STDIN (FORMAT csv)", "1,a,2\n2,,\n3,\"\",4\n"),
    (
        "from csv multiline",
        "COPY small FROM STDIN (FORMAT csv)",
        "1,\"line\nbreak, \"\"q\"\"\",3\n",
    ),
    ("from csv force not null", "COPY small FROM STDIN (FORMAT csv, FORCE_NOT_NULL (b))", "1,,2\n"),
    ("from csv force null", "COPY small FROM STDIN (FORMAT csv, FORCE_NULL (b))", "1,\"\",2\n"),
    ("from csv header match", "COPY small FROM STDIN (FORMAT csv, HEADER MATCH)", "a,b,c\n1,x,2\n"),
    (
        "from csv header true",
        "COPY small FROM STDIN (FORMAT csv, HEADER true)",
        "anything\n1,x,2\n",
    ),
    ("from csv end marker", "COPY small FROM STDIN (FORMAT csv)", "1,a,2\n\\.\n"),
    (
        "from csv delimiter quote",
        "COPY small FROM STDIN (FORMAT csv, DELIMITER ';', QUOTE '''')",
        "1;'a;b';2\n",
    ),
    (
        "from on error ignore",
        "COPY small FROM STDIN (ON_ERROR ignore)",
        "1\ta\t1\nx\tb\t2\n3\tc\ty\n4\td\t4\n",
    ),
    (
        "from on error verbose",
        "COPY small FROM STDIN (ON_ERROR ignore, LOG_VERBOSITY verbose)",
        "1\ta\t1\nx\tb\t2\n3\tc\ty\n",
    ),
    (
        "from on error silent",
        "COPY small FROM STDIN (ON_ERROR ignore, LOG_VERBOSITY silent)",
        "1\ta\t1\nx\tb\t2\n",
    ),
    (
        "from reject limit",
        "COPY small FROM STDIN (ON_ERROR ignore, REJECT_LIMIT 1)",
        "x\ta\t1\ny\tb\t2\n3\tc\t3\n",
    ),
    ("from latin1", "COPY small FROM STDIN (ENCODING 'LATIN1')", "1\tcaf\u{e9}\t1\n"),
    ("from error bad integer", "COPY small FROM STDIN", "1\ta\t1\nx\tb\t2\n"),
    ("from error missing data", "COPY small FROM STDIN", "1\n"),
    ("from error extra data", "COPY small FROM STDIN", "1\ta\t2\t3\n"),
    ("from error csv quote", "COPY small FROM STDIN (FORMAT csv)", "1,\"a,2\n"),
    (
        "from error header mismatch",
        "COPY small FROM STDIN (FORMAT csv, HEADER MATCH)",
        "a,x,c\n1,x,2\n",
    ),
    ("from error freeze", "COPY small FROM STDIN (FREEZE)", "1\ta\t1\n"),
    ("from error json", "COPY small FROM STDIN (FORMAT json)", "{}\n"),
    ("from error binary signature", "COPY small FROM STDIN (FORMAT binary)", "garbage\n"),
    ("from error reject limit", "COPY small FROM STDIN (REJECT_LIMIT 1)", "1\ta\t1\n"),
    ("from error default null", "COPY small FROM STDIN (DEFAULT '\\N')", "1\ta\t1\n"),
    ("from error invalid utf8", "COPY small FROM STDIN", "1\t\u{1}\u{ff}\t1\n"),
];

/// One case: the name, the statements before the copy, the copy, its data, and the query after it.
#[derive(Clone, Debug)]
pub(crate) struct Case {
    pub(crate) name: String,
    before: Option<&'static str>,
    copy: String,
    data: Option<Vec<u8>>,
    after: Option<&'static str>,
}

/// The cases of the suite. A round trip case gets its data from the oracle.
pub(crate) fn cases(oracle: &mut Conn) -> Result<Vec<Case>, String> {
    let mut out: Vec<Case> = OUT
        .iter()
        .map(|(name, sql)| Case {
            name: (*name).into(),
            before: None,
            copy: (*sql).into(),
            data: None,
            after: None,
        })
        .collect();
    for (name, to, from) in ROUND_TRIP {
        out.push(Case {
            name: (*name).into(),
            before: Some("TRUNCATE dst"),
            copy: (*from).into(),
            data: Some(copy_data(&oracle.query(to)?)),
            after: Some("COPY (SELECT * FROM dst ORDER BY id) TO STDOUT"),
        });
    }
    for (name, sql, data) in IN {
        // The text of the case is UTF-8. The LATIN1 case and the invalid UTF-8 case need other bytes.
        let bytes: Vec<u8> = if name.contains("latin1") || name.contains("invalid utf8") {
            data.chars().map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?')).collect()
        } else {
            data.as_bytes().to_vec()
        };
        out.push(Case {
            name: (*name).into(),
            before: Some("TRUNCATE small"),
            copy: (*sql).into(),
            data: Some(bytes),
            after: Some("COPY (SELECT * FROM small ORDER BY a NULLS FIRST, b, c) TO STDOUT"),
        });
    }
    Ok(out)
}

/// The bytes of the `CopyData` messages.
fn copy_data(msgs: &[Msg]) -> Vec<u8> {
    msgs.iter()
        .filter(|m| m.name() == "CopyData")
        .filter_map(|m| m.vals.first().and_then(Val::bytes))
        .flatten()
        .copied()
        .collect()
}

/// The messages in a form that compares. The `CopyData` messages of a copy are one line with all the bytes, because a server can cut the data in other places. Bytes that are not UTF-8 are in hex.
pub(crate) fn render(msgs: &[Msg]) -> String {
    let mut out = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    let flush = |data: &mut Vec<u8>, out: &mut Vec<String>| {
        if !data.is_empty() {
            let text = match std::str::from_utf8(data) {
                Ok(s) => format!("{s:?}"),
                Err(_) => data.iter().map(|b| format!("{b:02x}")).collect(),
            };
            out.push(format!("CopyData {text}"));
            data.clear();
        }
    };
    for m in msgs {
        match m.name() {
            "CopyData" => data.extend(m.vals.first().and_then(Val::bytes).unwrap_or_default()),
            name => {
                flush(&mut data, &mut out);
                match name {
                    "ErrorResponse" | "NoticeResponse" => {
                        out.push(format!("{name} {}", client::error_text(m)))
                    }
                    _ => out.push(format!("{name} {:?}", m.vals)),
                }
            }
        }
    }
    flush(&mut data, &mut out);
    out.join("\n")
}

/// Runs a case on one side and returns the messages of the copy and of the query after it.
fn run_case(c: &mut Conn, case: &Case) -> Result<String, String> {
    if let Some(sql) = case.before {
        c.execute(sql)?;
    }
    let msgs = match &case.data {
        Some(data) => c.copy_in(&case.copy, data)?,
        None => c.query(&case.copy)?,
    };
    let mut text = render(&msgs);
    if let Some(sql) = case.after {
        text.push_str("\n-- after\n");
        text.push_str(&render(&c.query(sql)?));
    }
    Ok(text)
}

/// Makes the database of the suite on one side.
pub(crate) fn prepare(target: &Target, db: &str) -> Result<Conn, String> {
    client::reset_database(target, db)?;
    let mut c = Conn::connect(target, db)?;
    for sql in SESSION.iter().chain(SETUP) {
        c.execute(sql)?;
    }
    Ok(c)
}

/// The result of one case.
#[derive(Clone, Debug)]
pub(crate) struct CaseResult {
    pub(crate) name: String,
    pub(crate) oracle: String,
    pub(crate) server: String,
}

/// Runs each case on the oracle and on the server under test. When the server under test is the oracle, the two sides use different databases.
pub(crate) fn run(oracle: &Target, server: &Target) -> Result<Vec<CaseResult>, String> {
    let server_db = if oracle.addr == server.addr { "copy_server" } else { DATABASE };
    let mut o = prepare(oracle, DATABASE)?;
    let list = cases(&mut o)?;
    let mut s = prepare(server, server_db);
    let mut out = Vec::new();
    for case in &list {
        let want = run_case(&mut o, case)?;
        let got = match &mut s {
            Ok(c) => run_case(c, case),
            Err(e) => Err(e.clone()),
        };
        let got = got.unwrap_or_else(|e| {
            // A lost connection fails this case, and the next case uses a new connection.
            s = Conn::connect(server, server_db).and_then(|mut c| {
                for sql in SESSION {
                    c.execute(sql)?;
                }
                Ok(c)
            });
            format!("no answer: {e}")
        });
        out.push(CaseResult { name: case.name.clone(), oracle: want, server: got });
    }
    Ok(out)
}

/// The first line of each answer that is not the same in the other answer.
pub(crate) fn first_lines<'a>(want: &'a str, got: &'a str) -> (&'a str, &'a str) {
    let mut w = want.lines();
    let mut g = got.lines();
    loop {
        match (w.next(), g.next()) {
            (Some(a), Some(b)) if a == b => {}
            (a, b) => return (a.unwrap_or("(no line)"), b.unwrap_or("(no line)")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Dir;

    #[test]
    fn copy_data_is_one_line() {
        let msgs = vec![
            Msg::new(Dir::B, "CopyOutResponse", vec![Val::Int(0), Val::List(vec![Val::Int(0)])]),
            Msg::new(Dir::B, "CopyData", vec![Val::Str(b"1\ta\n".to_vec())]),
            Msg::new(Dir::B, "CopyData", vec![Val::Str(b"2\tb\n".to_vec())]),
            Msg::new(Dir::B, "CopyDone", vec![]),
        ];
        let text = render(&msgs);
        assert_eq!(text.lines().nth(1), Some("CopyData \"1\\ta\\n2\\tb\\n\""));
        assert_eq!(text.lines().count(), 3);
        assert_eq!(copy_data(&msgs), b"1\ta\n2\tb\n");
    }

    #[test]
    fn bytes_that_are_not_utf8_are_in_hex() {
        let msgs = vec![Msg::new(Dir::B, "CopyData", vec![Val::Str(vec![0x50, 0xff])])];
        assert_eq!(render(&msgs), "CopyData 50ff");
    }

    #[test]
    fn the_first_different_lines() {
        assert_eq!(first_lines("a\nb\nc", "a\nx\nc"), ("b", "x"));
        assert_eq!(first_lines("a", "a\nb"), ("(no line)", "b"));
    }

    #[test]
    fn each_case_has_a_new_name() {
        let mut names: Vec<&str> = OUT.iter().map(|c| c.0).collect();
        names.extend(ROUND_TRIP.iter().map(|c| c.0));
        names.extend(IN.iter().map(|c| c.0));
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count);
    }
}
