#!/bin/sh
# The trace scenario for tokio-postgres. `rupg-compat record tokio-postgres` runs this script through the proxy.
# It builds the scenario with the pinned crates of Cargo.toml and Cargo.lock. The build is under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
export CARGO_TARGET_DIR="${RUPG_COMPAT_WORK:-work}/clients/rust"
cargo build --quiet --release --locked --manifest-path "$dir/Cargo.toml"
"$CARGO_TARGET_DIR/release/compat-client-tokio-postgres"
