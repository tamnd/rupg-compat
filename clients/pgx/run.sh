#!/bin/sh
# The trace scenario for pgx. `rupg-compat record pgx` runs this script through the proxy.
# With the argument gate, it builds and runs the gate program of the directory gate in place of the scenario, and `rupg-compat gate` runs it.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# It builds the program with the pinned module of go.mod. The module cache and the build cache are under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
work="${RUPG_COMPAT_WORK:-work}/clients/go"
export GOMODCACHE="$work/mod" GOCACHE="$work/cache" GOFLAGS=-modcacherw
if [ "${1:-}" = gate ]; then
    (cd "$dir" && go build -o "$work/pgx-gate" ./gate)
    "$work/pgx-gate" "$dir/../gate-queries.sql"
elif [ "${1:-}" = connect ]; then
    (cd "$dir" && go build -o "$work/pgx-connect" ./connect)
    "$work/pgx-connect"
else
    (cd "$dir" && go build -o "$work/pgx" .)
    "$work/pgx"
fi
