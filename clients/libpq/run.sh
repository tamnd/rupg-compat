#!/bin/sh
# The trace scenario for libpq. `rupg-compat record libpq` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# It builds the scenario with the libpq of the oracle build, so the version of libpq is the pin of the oracle. The build is under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record libpq}"
out="${RUPG_COMPAT_WORK:-work}/clients/libpq"
mkdir -p "$out"
include=$("$bin/pg_config" --includedir)
lib=$("$bin/pg_config" --libdir)
program="${1:-scenario}"
cc -std=c11 -Wall -Wextra -Werror -O2 -I"$include" -o "$out/$program" "$dir/$program.c" -L"$lib" -Wl,-rpath,"$lib" -lpq
"$out/$program"
