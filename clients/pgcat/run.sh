#!/bin/sh
# The trace scenario for pgcat. `rupg-compat record pgcat` runs this script through the proxy.
# It builds the pinned release from its source archive under the work directory and checks its SHA-256. A pooler is a client of the server. The scenario starts pgcat in transaction mode in front of the proxy, runs the psql scenario through it, then a short pgbench run with prepared statements on two connections. The trace has what pgcat sends to the server.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record pgcat}"
version=1.2.0
sha256=ff53e2678f9483c21d610009892513be6b9fa2c06278685be34d96f73d5bf4f2
out="${RUPG_COMPAT_WORK:-work}/clients/pgcat-$version"
if [ ! -x "$out/bin/pgcat" ]; then
    mkdir -p "$out"
    curl -sfL -o "$out/src.tar.gz" "https://github.com/postgresml/pgcat/archive/refs/tags/v$version.tar.gz"
    echo "$sha256  $out/src.tar.gz" | sha256sum -c --quiet
    tar xzf "$out/src.tar.gz" -C "$out"
    CARGO_TARGET_DIR="$out/target" cargo install --quiet --locked --root "$out" --path "$out/pgcat-$version"
fi
# A free port on the loopback for pgcat.
port=$(python3 -I -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
cat > "$out/pgcat.toml" <<CONFIG
[general]
host = "127.0.0.1"
port = $port
admin_username = "pgcat"
admin_password = "pgcat"

[pools.$PGDATABASE]
pool_mode = "transaction"
prepared_statements_cache_size = 500

[pools.$PGDATABASE.users.0]
username = "$PGUSER"
password = "$PGPASSWORD"
pool_size = 2

[pools.$PGDATABASE.shards.0]
servers = [["$PGHOST", $PGPORT, "primary"]]
database = "$PGDATABASE"
CONFIG
"$out/bin/pgcat" "$out/pgcat.toml" > "$out/pgcat.log" 2>&1 &
pgcat=$!
trap 'kill $pgcat' EXIT
i=0
until "$bin/pg_isready" -q -h 127.0.0.1 -p "$port"; do
    i=$((i + 1))
    [ $i -lt 60 ] || { echo "pgcat did not start" >&2; exit 1; }
    sleep 1
done
export PGHOST=127.0.0.1 PGPORT="$port"
# 1. The psql scenario.
"$bin/psql" -X -q -f "$dir/../psql/scenario.sql"
# 2. pgbench with prepared statements on two connections. The server makes the rows of the tables, so the trace does not hold them.
"$bin/pgbench" -q -i -I dtGp -s 1
"$bin/pgbench" -n -c 2 -j 1 -t 20 -M prepared > /dev/null
"$bin/pgbench" -q -i -I d
