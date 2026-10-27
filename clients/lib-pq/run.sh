#!/bin/sh
# The trace scenario for lib-pq. `rupg-compat record lib-pq` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# It builds the scenario with the pinned module of go.mod. The module cache and the build cache are under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
work="${RUPG_COMPAT_WORK:-work}/clients/go"
export GOMODCACHE="$work/mod" GOCACHE="$work/cache" GOFLAGS=-modcacherw
if [ "${1:-}" = connect ]; then
    (cd "$dir" && go build -o "$work/lib-pq-connect" ./connect)
    "$work/lib-pq-connect"
else
    (cd "$dir" && go build -o "$work/lib-pq" .)
    "$work/lib-pq"
fi
