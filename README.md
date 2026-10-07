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

## The suites

| Suite | Size at the pin | Level | First milestone |
|---|---|---|---|
| Regression | 239 tests | L3 | M3 tracked, M7 full schedule, M12 at 100 percent |
| Isolation | 135 specs | L4 | M5 |
| `libpq_pipeline` | 9 traced tests | L1 | M2 |
| TAP subset | of 294 files | L4 and L5 | M9 |
| Contrib regression | 61 directories, 55 extensions | L3 | M7 |
| Client suites | 48 clients | L5 | M2 for the 7 gate clients |
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
cargo run --release -- client pgx
cargo run --release -- gen --oracle tlp --seed 42
cargo run --release -- reduce corpus/reduced/case.sql
cargo run --release -- bisect corpus/reduced/case.sql
cargo run --release -- tap
cargo run --release -- jepsen
cargo run --release -- report
```

Every command takes `--compat-version V`, which sets `rupg.compat_version` on rupg and picks the oracle of version V. `up` starts the servers, and `client`, `regress`, `isolation`, `tap` and `jepsen` each run one suite. Replay maps OIDs, runs a new SCRAM exchange for each server, and rewrites the key of each `CancelRequest`.

## The reporting rules

1. The oracle decides. When the documentation and the oracle disagree, the oracle is correct.
2. The regression number is the strict count: a test passes only when `pg_regress` says so. The statement count is published for progress and never used for a gate.
3. A test leaves the denominator only when it needs a `regress.c` function or a C extension that rupg does not have, or when its purpose is an excluded item such as the exact `EXPLAIN` output. The list is published with the reason for each test.
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
clients/<name>/    run.sh, oracle-fail.txt, expected-fail.txt
corpus/            traces, the differential corpus, reduced cases
shim/<N>/          divergences.toml for each shim version
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
