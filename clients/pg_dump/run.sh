#!/bin/sh
# The trace scenario for pg_dump and pg_restore. `rupg-compat record pg_dump` runs this script through the proxy.
# The programs are the ones of the oracle build, so their version is the pin of the oracle. The scenario makes a schema with psql, dumps it in the custom format, drops it, restores it with pg_restore and dumps it again as text.
set -eu
dir=$(dirname "$0")
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record pg_dump}"
out="${RUPG_COMPAT_WORK:-work}/clients/pg_dump"
mkdir -p "$out"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
"$bin/pg_dump" -Fc -f "$out/compat.dump"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP SCHEMA public CASCADE' -c 'CREATE SCHEMA public'
"$bin/pg_restore" --exit-on-error -d "$PGDATABASE" "$out/compat.dump"
"$bin/pg_dump" -f "$out/compat.sql"
