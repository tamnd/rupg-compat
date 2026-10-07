//! The PostgreSQL compatibility harness for rupg.
//!
//! The runner sends one input to a PostgreSQL oracle and to rupg and compares the two answers. The levels L1 to L5 and the suites are in `spec/05-compatibility.md` and `spec/21-testing.md` of tamnd/rupg. At M0 the binary prints the oracle pins and the levels.

#![forbid(unsafe_code)]

use std::process::ExitCode;

mod levels;
mod oracle;

const USAGE: &str = "usage: rupg-compat <pins | levels | --version>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("pins") => {
            for pin in oracle::PINS {
                println!(
                    "{:<4} {:<10} {:<20} {}",
                    pin.major,
                    pin.release,
                    pin.git_ref,
                    pin.commit.unwrap_or("tag")
                );
            }
            ExitCode::SUCCESS
        }
        Some("levels") => {
            for level in levels::LEVELS {
                println!("{}  {}", level.id, level.name);
            }
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("rupg-compat {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
