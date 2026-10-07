#!/bin/sh
# The trace scenario for pgx. `rupg-compat record pgx` runs this script through the proxy.
# It builds the scenario with the pinned module of go.mod. The module cache and the build cache are under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
work="${RUPG_COMPAT_WORK:-work}/clients/go"
export GOMODCACHE="$work/mod" GOCACHE="$work/cache" GOFLAGS=-modcacherw
(cd "$dir" && go build -o "$work/pgx" .)
"$work/pgx"
