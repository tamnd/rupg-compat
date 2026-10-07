# Contributing

This repository holds the PostgreSQL oracles, the runner and the suites that decide the compatibility levels of rupg. The engine is in [tamnd/rupg](https://github.com/tamnd/rupg).

## Before you start

Read [`spec/05-compatibility.md`](https://github.com/tamnd/rupg/blob/main/spec/05-compatibility.md) and [`spec/21-testing.md`](https://github.com/tamnd/rupg/blob/main/spec/21-testing.md) in the engine repository. They define the levels, the denominators and the excluded bytes.

## The rules for a number

1. The oracle is right by definition. It is a PostgreSQL server built from a pin, not the documentation.
2. Each level has a denominator that is counted at the pin. A change to a denominator gives the old count, the new count and how you counted.
3. A test that rupg fails stays in the denominator. Do not remove a failing test to raise a pass rate.
4. Each difference that is allowed is in the exclusion list with a reason. A difference that is not in the list is a failure.
5. The L5 number is a number for each client. Do not add the clients into one sum.

## Running the checks

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features
cargo test
```

## Writing style

The README, the reports and the issues use ASD-STE100 technical English. Write short sentences in the active voice. Use one line for each paragraph. Do not use the em dash or the en dash.

## License

By contributing, you agree that your contribution is licensed under Apache-2.0. Upstream suites keep their own licenses in their own directories. Do not copy code under the AGPL or another copyleft license into this repository.
