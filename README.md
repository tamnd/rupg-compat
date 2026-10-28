# rupg-compat

The PostgreSQL compatibility harness for [rupg](https://github.com/tamnd/rupg).

[![ci](https://github.com/tamnd/rupg-compat/actions/workflows/ci.yml/badge.svg)](https://github.com/tamnd/rupg-compat/actions/workflows/ci.yml) [![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE-APACHE)

This harness turns "compatible with PostgreSQL" from a claim into a number that a machine computed. It builds real PostgreSQL servers from pinned commits, sends the same input to an oracle and to rupg, and compares every byte that a client can see. It runs the PostgreSQL regression and isolation suites, the test suites of 48 clients, sqllogictest and generated queries, and it publishes one number for each level.

It is a separate repository for three reasons. It must build and run PostgreSQL, which no crate that ships with rupg should do. It does not link a rupg crate, so a bug in the engine's codec cannot be on both sides of the comparison. And a suite that lives inside the code it tests is a suite whose failures are easy to explain away.

The design is [`spec/21-testing.md`](https://github.com/tamnd/rupg/blob/main/spec/21-testing.md#213-the-harness) section 21.3, and the levels it reports are [`spec/05-compatibility.md`](https://github.com/tamnd/rupg/blob/main/spec/05-compatibility.md#51-what-compatible-means) section 5.1.

## Status

Early. The crate builds and CI is green, but no suite runs yet. The pins are in `pins.toml`. `rupg-compat pins` prints them and `rupg-compat levels` prints the five levels. The first milestone, M0, builds the oracles and the runner, and it closes when the harness scores 100 percent against the oracle of 19. Every command below is the planned interface from the spec.

## The principle

The oracle is a real PostgreSQL server built from the pin. It is correct by definition, also where it disagrees with the documentation. When the oracle gives an answer that depends on the run, for example the order of rows without `ORDER BY`, the harness runs the oracle twice and removes the unstable case. A test may not add an exclusion. The closed list of 15 excluded items is in spec/05 section 5.3, and the report prints each exclusion that a run used, with its count.

## The five levels

| Level | Name | Denominator at the pin |
|---|---|---|
| L1 | Connect | 52 message formats, the 9 trace files of `libpq_pipeline`, the protocol traces of 48 clients |
| L2 | Introspect | 64 catalogs (11 shared), 86 system views, 65 `information_schema` views, and the catalog corpus from the recording proxy |
| L3 | Query | 239 regression tests with 290,977 lines of expected output, the differential corpus, the 3,414 rows of `pg_proc` |
| L4 | Behave | 135 isolation specs (133 scheduled), Hermitage, 7,112 `ERROR` lines in 191 files, 438 configuration parameters |
| L5 | Drop-in | the upstream test suites of the 48 clients, counted for each client and never added into one sum |

A level is at 100 percent when every item of its denominator gives the same result on rupg as on the oracle, after the exclusions.

## The oracles

| Version | Pin |
|---|---|
| 19 | `REL_19_STABLE` at `7d3d2db7` (19beta4), then RC1 (15 October 2026), then the 19.0 tag (29 October 2026) |
| 18 | 18.6 |
| 17 | 17.11 |
| 16 | 16.15 |
| 15 | 15.19 |
| 14 | 14.24, kept until 12 November 2027, one year after its end of life |

Every oracle runs with the same fixed configuration: `initdb --no-locale` with `UTF8`, `TimeZone` UTC, `DateStyle` `ISO, MDY`, `lc_messages` C, SCRAM passwords, and TLS with a fixed test certificate. A second nightly set runs with an ICU locale and a time zone that is not UTC. The 19 pin moves on a branch and lands on one day, with every count taken again.

The server under test runs with the same configuration files as the oracle. `rupg-compat oracle conf DIR --port N` copies `postgresql.conf`, the host rules, the certificate and its key from the data directory of the oracle to DIR, and changes only the port. Then `rupg serve -D DIR` reads them. Without these files, `pg_settings`, `pg_file_settings`, `pg_hba_file_rules`, `ssl` and `max_prepared_transactions` give other values than the oracle, and the cause is the setup and not the server.

## The suites

| Suite | Size at the pin | Level | First milestone |
|---|---|---|---|
| Regression | 239 tests | L3 | M3 tracked, M7 full schedule, M12 at 100 percent |
| Isolation | 135 specs | L4 | M5 |
| sqllogictest | 622 files of SQLite, 30 files each night | L3 | M3 |
| `libpq_pipeline` | 9 traced tests | L1 | M2 |
| `pg_dump` round trip | 7 checks for each oracle version | L2 and L5 | M2 |
| `COPY` | 67 cases in the text, csv, binary and json formats | L3 | M4 |
| TAP subset | of 294 files | L4 and L5 | M9 |
| Contrib regression | 61 directories, 55 extensions | L3 | M7 |
| Client suites | 48 clients | L5 | M2 for the 7 gate clients |
| Gate G1 | 7 clients show the gate schema | L1 and L2 | M2 |
| Connect | psql, libpq and 15 drivers with SCRAM over TLS | L1 | M2 |
| Version forms | 8 checks | L1 | M2 |
| Hermitage | at three isolation levels | L4 | M5 |
| sqllogictest | the SQLite corpus, answers from the oracle | L3 | M3 |
| Generated queries | 3,476 `gram.y` productions, five oracles | L3 | M3 |
| Jepsen and Elle | one server from M5, 3 to 16 nodes from M10 | L4 | M5 |

The generator checks each query with one of five oracles: differential, TLP, NoREC, PQS and layout equivalence over five layouts of the same data. Each shim version from 14 to 18 runs its own suites against its own oracle, at the cadence of spec/05 section 5.6.5.

## Running it

The commands below are the planned interface. The harness is one Rust crate with a binary named `rupg-compat`, built with Rust 1.98.0.

```
cargo run --release -- oracle build --version 19
cargo run --release -- record psql
cargo run --release -- replay corpus/traces/psql.trace
cargo run --release -- diff "SELECT 1"
cargo run --release -- regress
cargo run --release -- isolation
cargo run --release -- dump --server 127.0.0.1:5432
cargo run --release -- gate --server 127.0.0.1:5432
cargo run --release -- connect --server 127.0.0.1:5432
cargo run --release -- version --server 127.0.0.1:5432
cargo run --release -- client pgx
cargo run --release -- gen --oracle tlp --seed 42
cargo run --release -- reduce corpus/reduced/case.sql
cargo run --release -- bisect corpus/reduced/case.sql
cargo run --release -- tap
cargo run --release -- jepsen
cargo run --release -- report
cargo run --release -- reference --check
```

Every command takes `--compat-version V`, which sets `rupg.compat_version` on rupg and picks the oracle of version V. `up` starts the servers, and `client`, `regress`, `isolation`, `tap` and `jepsen` each run one suite. Replay maps OIDs, runs a new SCRAM exchange for each server, and rewrites the key of each `CancelRequest`.

Every command also takes `--cgroup DIR`, the cgroup v2 directory of the server that the run measures. Each result file then has the peak memory (`memory.peak`) and the CPU time (`usage_usec` of `cpu.stat`) of that server, as spec/02 section 2.6.1 says, and the report shows the ratio of the oracle to rupg for each suite. Before Linux 6.12 the kernel cannot reset `memory.peak`, so give each suite a new cgroup to get the peak of that suite. The report also lists the failing tests of each run of the server under test. `report --rupg-commit SHA` writes the commit of rupg under test in the pins table, because the pins of the binary can be older.

Six of the 48 clients are closed source or run only as a hosted service: DataGrip, TablePlus, Postico 2, Tableau, Power BI and Looker. CI cannot run them, so `report` shows each of them in L1 with the numbers of the driver that it uses: pgjdbc for DataGrip and Looker, Npgsql for Power BI, psqlODBC for Tableau, and libpq for TablePlus and Postico 2. These rows have no count of their own, and the ratchet checks only the row of the driver. A trace that a person records of one of these tools replaces the driver row. DBeaver and Beekeeper Studio have no headless mode, so `clients/dbeaver` and `clients/beekeeper` send the queries of their PostgreSQL code at the pinned release through pgjdbc and node-postgres. Replay also maps an OID in a `text` column with the name `oid`, because Beekeeper Studio casts the OID of a function to text and sends it back to `pg_get_functiondef`.

`catalog` runs `SELECT *` on each catalog and each view of L2 in a new database, two times on the oracle and one time on the server under test, and compares the rows. With `--schema gate`, it first runs `clients/gate.sql` in the new database of each server, so the rows of the gate schema are compared too. That run is the suite `catalog-gate`. The report shows the rows of L2 tier 1 and tier 2 apart (spec/05 section 5.1). Tier 1 is `pg_type`, `pg_namespace`, `pg_class` and `pg_attribute`. Tier 2 is `pg_index`, `pg_constraint`, `pg_attrdef`, `pg_description`, `pg_enum`, `pg_range`, `pg_proc` and each relation of `information_schema`. Each tier row is the sum of the equal rows of its relations, of the sum of their rows.

`catalog-corpus` is the catalog corpus that the recording proxy captured from the clients (spec/05 section 5.6.5). It reads the traces in `corpus/traces` and keeps each query that starts with `SELECT` or `WITH`, reads the catalog, changes nothing and has no parameters. Each query is in the suite one time. The suite runs `clients/gate.sql` in a new database, runs the queries two times on the oracle and one time on the server under test, and compares the answers with the rules of the catalog suite. The name of a case is the name of the first trace that sends the query and the number of the query in that trace.

`types` checks the four functions of each type (spec/07 section 7.9 of rupg). For each value of `suites/types/values.txt`, it runs three cases: `SELECT 'v'::T` as a simple query for the text output, the same query with a binary result for the send function, and `SELECT $1` with the binary value of the oracle as a parameter of type T for the receive function. A value that the oracle rejects has only the first two cases, so the error is compared too. The report shows the suite in L3.

The oracles are built with lz4, because PostgreSQL 19 then makes lz4 the default of `default_toast_compression`, as the spec of rupg gives. The compression changes which rules of the system views take a TOAST value, so it changes the OIDs of the objects that `initdb` makes.

The oracles are also built with zstd, because rupg has zstd. With zstd, `wal_compression` accepts `zstd`, and `pg_settings` shows it in `enumvals`.

`dump` runs `clients/pg_dump/roundtrip.sql` in a new database of the server under test and dumps it with `pg_dump` of the oracle build. Then it restores the dump into the oracle with `psql` and with `pg_restore`, dumps the oracle again, and compares the texts. It also compares the dump of the server under test with the dump of the oracle after the same script, and restores the dump of the oracle into the server under test. The comparison ignores the line `-- Dumped from database version` and the random key of `\restrict`.

The workflow `nightly` builds rupg main each night and runs `.github/nightly.sh`, which runs each suite on the oracle and on rupg and then `report` (spec/05 section 5.8). Each server runs in a new cgroup v2 for each suite, so the report has the peak memory and the CPU time of each suite also on a kernel before Linux 6.12. A second job pushes the report to `commits/<commit>.md` on the branch `pages`, with a line in its `README.md`. The job with the suites has a read-only token, and only the job that pushes can write. The result files and the logs stay as an artifact for 14 days. `workflow_dispatch` takes `rupg-ref` to test another commit.

`sqllogictest` fetches the sqllogictest corpus of SQLite at the commit of `[sqllogictest]` in `pins.toml` into the work directory. It runs each record of the test files under the given paths on the oracle and on the server under test, and compares the answers of the two servers. It does not use the expected answers of the corpus, because they are the answers of SQLite. A record that `skipif postgresql` or `onlyif` with another engine names does not run. A statement gives its command tag or its error. A query gives its rows in the order of its sort mode, or its error. A file passes when each record gives the answer of the oracle, and the report has the rows "sqllogictest files" and "sqllogictest records" at L3. Each worker uses the database `slt_<n>` on each server. Without `--server`, the oracle runs against itself in a second database. The nightly runs `select1.test` to `select5.test`, `evidence` and `index/between`, which are 135,648 records.

`generate` reads `gram.y` and `kwlist.h` of the oracle source at the pin, which have 743 rules and 3,476 productions, and writes statements that start from `SelectStmt`, `InsertStmt`, `UpdateStmt` or `DeleteStmt`. The seed (`--seed N`, else the clock) makes a schema of three tables with data, and then each statement, so the same seed gives the same run. The seed is on the first line and on the last line of the output. Each statement runs in a transaction that rolls back, on two databases of the oracle and on the server under test. A statement that does not give the same answer on the two oracle databases is unstable and does not count. A choice of a production uses its weight, and after each 200 statements the weight of a production that no statement used goes up to two times its value, to at most 64. Past a depth of 9 or 80 tokens, each choice takes a production with the smallest expansion. The output gives the productions that the run used. The statements that are different go to `generate/<seed>.sql` in the work directory, with the schema and the two answers, and the report has the row "generated statements" at L3. Most of the statements give an error on the oracle, so this is the differential oracle for the parser and for the order of the checks. The layout oracle follows.

`reduce FILE` makes a SQL file that fails on the server under test smaller, for example a file of `generate/` or `logic/` in the work directory. It runs the statements of the file in order in a new database on the oracle and on the server under test. The way the file fails is the first statement with a different answer, and for that statement the first different message type and the SQLSTATE on each side. The first pass removes statements with delta debugging. The second pass removes clauses, groups in parentheses, items of lists and single tokens, and puts the content of a group or `NULL` in place of the group. The third pass puts `0`, `1`, `''` or `'a'` in place of a literal. A step stays only if the file still fails in the same way, so that the reduction does not move to a different bug. The result goes to `FILE.reduced.sql` (or `--out FILE`), with the way it fails on the first line. `--max-runs N` (default 3,000) stops the reduction after N runs of the file.

`copy` runs 67 cases of `COPY` in a new database on the oracle and on the server under test, and compares the messages of each case. The table `src` has 15 columns of common types and rows with special values: NULL, NaN, infinity, a tab, a new line, a backslash, `\N`, the text `NULL`, an empty text and text that is not ASCII. There are 30 `COPY TO` cases for each format and its options, for a query, for a column list, for `ENCODING` and for the errors of the options. There are 7 round trip cases: the oracle writes `src` in one format, and each side reads that output into `dst`. There are 30 `COPY FROM` cases with fixed data, for the escapes, the end marker, `WHERE`, `DEFAULT`, `FORCE_NOT_NULL`, `FORCE_NULL`, `HEADER MATCH`, `ON_ERROR`, `LOG_VERBOSITY`, `REJECT_LIMIT`, `ENCODING` and the errors. A `COPY FROM` case compares the messages of the copy and then the rows of the table. The `CopyData` messages of a copy compare as one string, because a server can cut the data in other places. The report has the row "COPY cases" at L3.

`logic` uses the schema and the data of the seed, as `generate` does, and makes a predicate over the columns of one table or of two joined tables for each check. The type of each expression is known, so that most predicates are correct. TLP checks that the rows of a query are the rows where the predicate is true, false or NULL, for `SELECT *`, `SELECT DISTINCT *` and `count(*)`. NoREC checks that `count(*)` with the predicate in `WHERE` is the sum of `(p) IS TRUE` over all the rows. PQS asks the oracle for the value of a predicate on a row, changes the predicate so that it is true on that row, and checks that the row is in the answer of the server. A check is first done on the oracle, and it is skipped when a query gives an error there. The differential check compares the answer of each query on the server with the answer of the oracle. The failed checks go to `logic/<seed>.sql` in the work directory, and the report has the row "logic checks" at L3. The nightly runs 5,000 predicates with the date as the seed.

`gate` runs `clients/gate.sql` in a new database of the oracle and of the server under test. Then it runs `clients/<name>/run.sh gate` for psql, pgAdmin 4, pgjdbc, psycopg, node-postgres, pgx and asyncpg. Each gate program lists the tables, the columns, the types, the indexes and the constraints of the schema `gate`. psql uses its describe commands, pgAdmin 4 walks its browser tree and writes the SQL tab of each object, and pgjdbc writes the results of `DatabaseMetaData`. The other drivers have no describe command, so they run the catalog queries of `clients/gate-queries.sql` and write the columns that they describe for a query of each table. A client passes when its list on the server under test is equal to its list on the oracle. The test of each client is `gate` in its `oracle-fail.txt` and `expected-fail.txt`. The gate does not run a client whose test is in `oracle-fail.txt`. The command exits with 1 when a client fails that is not in `expected-fail.txt`, or when a client in that list passes. `pipeline --server` uses the lists in `suites/pipeline` in the same way, with the name of each test and each trace as the test.

`connect` runs `clients/<name>/run.sh connect` for psql, libpq and the 15 drivers: pgjdbc, psycopg, psycopg2, asyncpg, node-postgres, postgres.js, pgx, lib/pq, Npgsql, tokio-postgres, sqlx, the Ruby pg gem, Postgrex, the R2DBC driver and psqlODBC. The Rust programs are crates of their own in `connect/`, so the trace scenarios keep their lock files and build without TLS. The proxy has no TLS, so the traces cannot show the TLS handshake. This suite does that check. Each program connects to the database `postgres` with TLS, checks the certificate of the oracle and requires channel binding where the client can do it. Then it runs one query that gives `system_user`, the TLS flag and the TLS version of its own connection, writes one line and disconnects. The runner writes the certificate of the oracle to `root.crt` in the run directory and sets `PGSSLMODE=verify-full`, `PGSSLROOTCERT` and `PGCHANNELBINDING=require`. A client passes when its line on the server under test is equal to its line on the oracle. The test of each client is `connect` in its lists.

`version` connects to the server under test and checks the four places where rupg gives its version, as spec/05 section 5.7 says. `server_version` at startup and `SHOW server_version` must be `N.M (rupg X.Y.Z)`, where N is the compat version. `server_version_num` must be `N0M` in the PostgreSQL form, and `version()` must be `PostgreSQL N.M (rupg X.Y.Z) on <target>, compiled by rustc <version>, 64-bit`. `rupg.version` at startup and `SHOW rupg.version` must be `X.Y.Z`, and `SET rupg.version` must fail with `55P02`, as for a parameter with the context `internal`. The server must send `rupg.version` after the standard parameters of version N, and it must not send `crdb_version`, `mz_version` or `questdb_version`. The oracle does not have these forms, so the command needs `--server`.

## The reporting rules

1. The oracle decides. When the documentation and the oracle disagree, the oracle is correct.
2. The regression number is the strict count: a test passes only when `pg_regress` says so. The statement count is published for progress and never used for a gate.
3. A test leaves the denominator only when it needs a `regress.c` function or a C extension that rupg does not have, or when its purpose is an excluded item such as the exact `EXPLAIN` output. The list is published with the reason for each test in `suites/regress/excluded.txt`, and the report shows it.
4. `expected-fail.txt` for each client may only get shorter, and each line names its cause and the milestone that fixes it.
5. `ratchet.toml` holds the best number of each row, and `rupg-compat report` fails when a number goes below it. A number may go down only with an entry in spec/24 that says why.
6. The number can go down when the pin moves. The fall is published with its cause, and the pin is never held back to keep a number.
7. A difference that a later PostgreSQL release fixed is recorded as "fixed upstream" and is not counted against a shim.
8. One page is published for each commit on `main`, with the L1 to L5 numbers for 19, the shim numbers, the failing items, and the peak memory and CPU time of rupg and of the oracle for each suite.

## Layout

```
pins.toml          PostgreSQL commits, client versions, the rupg commit
oracle/            build scripts for the six oracles
src/               the runner, the proxy, the comparison, the generator
clients/<name>/    run.sh, the gate and connect programs, oracle-fail.txt, expected-fail.txt
suites/pipeline/   oracle-fail.txt, expected-fail.txt for libpq_pipeline
suites/regress/    excluded.txt, the tests that leave the counts of the regression suite, milestones.txt, the milestone of each other test, and explain.txt, the tests whose EXPLAIN lines are filtered
corpus/            traces, the differential corpus, reduced cases
shim/<N>/          divergences.toml, and the reference output of the oracle of N in reference/
jepsen/            the Jepsen and Elle workloads
ratchet.toml       the best number of each row
reports/<date>/    one table for each level, and each shim version
```

## What this harness cannot do

A suite can show that a bug exists, not that no bug exists. The numbers say that the suites found no difference after a stated effort, and the release report states the effort in hours and cases. The harness does not test the speed of rupg. That is [rupg-bench](https://github.com/tamnd/rupg-bench).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md). The milestones are issues with the label `kind/milestone`.

## License

Apache-2.0. See [LICENSE-APACHE](LICENSE-APACHE). This project is not affiliated with the PostgreSQL Global Development Group.
