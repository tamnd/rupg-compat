#!/bin/sh
# The trace scenario for tokio-postgres. `rupg-compat record tokio-postgres` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# It builds the scenario with the pinned crates of Cargo.toml and Cargo.lock. The build is under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
export CARGO_TARGET_DIR="${RUPG_COMPAT_WORK:-work}/clients/rust"
if [ "${1:-}" = connect ]; then
    cargo build --quiet --release --locked --manifest-path "$dir/connect/Cargo.toml"
    "$CARGO_TARGET_DIR/release/compat-client-tokio-postgres-connect"
else
    cargo build --quiet --release --locked --manifest-path "$dir/Cargo.toml"
    "$CARGO_TARGET_DIR/release/compat-client-tokio-postgres"
fi
