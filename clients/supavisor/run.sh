#!/bin/sh
# The trace scenario for Supavisor. `rupg-compat record supavisor` runs this script through the proxy.
# It builds the pinned release from its source archive under the work directory and checks its SHA-256, with the Erlang/OTP and Elixir of elixir.sh and the cargo of the host for its Rust NIF. A pooler is a client of the server. Supavisor keeps its tenants in a metadata database, so the script starts a second PostgreSQL with the binaries of the oracle build for it. That server is not behind the proxy, so the trace has only what Supavisor sends to the oracle for the tenant. The scenario adds one tenant in transaction mode with named prepared statements on (tenant.py says how when Supavisor rejects the version of the server), runs the psql scenario through Supavisor, then a short pgbench run with prepared statements on two connections.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record supavisor}"
version=2.9.13
sha256=ec071eab399163b56f5c406d8a6673180f1f00de1b5b07bb2b8d047e4cae7646
. "$dir/../elixir.sh"
out="${RUPG_COMPAT_WORK:-work}/clients/supavisor-$version"
mkdir -p "$out"
out=$(cd "$out" && pwd)
src="$out/supavisor-$version"
export MIX_ENV=prod
if [ ! -f "$out/built" ]; then
    curl -sfL -o "$out/src.tar.gz" "https://github.com/supabase/supavisor/archive/refs/tags/v$version.tar.gz"
    echo "$sha256  $out/src.tar.gz" | sha256sum -c --quiet
    rm -rf "$src"
    tar xzf "$out/src.tar.gz" -C "$out"
    # bindgen of the pg_query crate needs stddef.h, and libclang finds it only with the include directory of the compiler.
    export BINDGEN_EXTRA_CLANG_ARGS="-I$(cc -print-file-name=include)"
    (cd "$src" && mix local.hex "$hex_version" --force --if-missing >/dev/null && mix local.rebar --force >/dev/null && mix deps.get --only prod --check-locked >/dev/null 2>&1 && mix compile >/dev/null 2>&1)
    touch "$out/built"
fi
# A free port on the loopback for each listener.
ports=$(python3 -I -c '
import socket
socks = [socket.socket() for _ in range(13)]
for s in socks:
    s.bind(("127.0.0.1", 0))
print(" ".join(str(s.getsockname()[1]) for s in socks))
')
set -- $ports
meta_port=$1 http_port=$2 session_port=$3 transaction_port=$4 proxy_port=$5
session_ports="$6,$7,$8,$9"
shift 9
transaction_ports="$1,$2,$3,$4"
# The metadata database: a new cluster for each run.
meta="$out/meta"
rm -rf "$meta" "$out/supavisor.log" "$out/tenant.json"
"$bin/initdb" -A trust -U postgres -D "$meta" > "$out/initdb.log"
"$bin/pg_ctl" -s -D "$meta" -l "$out/meta.log" -o "-p $meta_port -k $meta -c listen_addresses=127.0.0.1" -w start
supavisor=
stop() {
    [ -z "$supavisor" ] || kill "$supavisor" 2>/dev/null || true
    "$bin/pg_ctl" -s -D "$meta" -m fast -w stop || true
}
trap stop EXIT
export DATABASE_URL="ecto://postgres@127.0.0.1:$meta_port/postgres" DB_POOL_SIZE=2
export VAULT_ENC_KEY=compat-scenario-vault-key-32byte API_JWT_SECRET=compat-api METRICS_JWT_SECRET=compat-metrics
export SECRET_KEY_BASE=compat-scenario-secret-key-base-compat-scenario-secret-key-base-0
export REGION=local NODE_IP=127.0.0.1 PORT="$http_port" PROXY_PORT_SESSION="$session_port" PROXY_PORT_TRANSACTION="$transaction_port" PROXY_PORT="$proxy_port"
export SESSION_PROXY_PORTS="$session_ports" TRANSACTION_PROXY_PORTS="$transaction_ports" NAMED_PREPARED_STATEMENTS_ENABLED=true
cd "$src"
mix run --no-compile --no-start -e 'Supavisor.Release.migrate()' > "$out/migrate.log" 2>&1
mix run --no-compile --no-halt > "$out/supavisor.log" 2>&1 &
supavisor=$!
status=0
python3 -I "$dir/tenant.py" "$http_port" "$out/tenant.json" || status=$?
if [ "$status" = 3 ]; then
    mix run --no-compile --no-start "$dir/tenant.exs" "$out/tenant.json" >> "$out/migrate.log" 2>&1
elif [ "$status" != 0 ]; then
    exit "$status"
fi
export PGHOST=127.0.0.1 PGPORT="$transaction_port" PGUSER="$PGUSER.compat"
# 1. The psql scenario.
"$bin/psql" -X -q -f "$dir/../psql/scenario.sql"
# 2. pgbench with prepared statements on two connections. The server makes the rows of the tables, so the trace does not hold them.
"$bin/pgbench" -q -i -I dtGp -s 1
"$bin/pgbench" -n -c 2 -j 1 -t 20 -M prepared > /dev/null
"$bin/pgbench" -q -i -I d
