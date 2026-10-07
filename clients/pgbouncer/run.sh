#!/bin/sh
# The trace scenario for PgBouncer. `rupg-compat record pgbouncer` runs this script through the proxy.
# It builds the pinned release and libevent from their source archives under the work directory and checks their SHA-256. The build has no TLS, because the proxy does not speak TLS. A pooler is a client of the server. The scenario starts PgBouncer in transaction mode with prepared statements in front of the proxy, runs the psql scenario through it, then a short pgbench run with prepared statements on two connections. The trace has what PgBouncer sends to the server.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record pgbouncer}"
version=1.26.0
sha256=afd25dd61ee6775d37b40629b87ce08736b3e6955f3057bb212e410fbf21c71d
libevent=2.1.13-stable
libevent_sha256=f7e9383b8c0baa81b687e5b5eecc01beefaf1b19b64151d95ed61647fe7a315c
out="${RUPG_COMPAT_WORK:-work}/clients/pgbouncer-$version"
mkdir -p "$out"
out=$(cd "$out" && pwd)
if [ ! -x "$out/pgbouncer-$version/pgbouncer" ]; then
    curl -sfL -o "$out/libevent.tar.gz" "https://github.com/libevent/libevent/releases/download/release-$libevent/libevent-$libevent.tar.gz"
    echo "$libevent_sha256  $out/libevent.tar.gz" | sha256sum -c --quiet
    curl -sfL -o "$out/pgbouncer.tar.gz" "https://github.com/pgbouncer/pgbouncer/releases/download/pgbouncer_$(echo $version | tr . _)/pgbouncer-$version.tar.gz"
    echo "$sha256  $out/pgbouncer.tar.gz" | sha256sum -c --quiet
    tar xzf "$out/libevent.tar.gz" -C "$out"
    tar xzf "$out/pgbouncer.tar.gz" -C "$out"
    (cd "$out/libevent-$libevent" && ./configure -q --prefix="$out/libevent" --disable-openssl --disable-shared --disable-samples --disable-libevent-regress && make -s && make -s install) > "$out/build.log" 2>&1
    (cd "$out/pgbouncer-$version" && LIBEVENT_CFLAGS="-I$out/libevent/include" LIBEVENT_LIBS="$out/libevent/lib/libevent.a" ./configure -q --without-openssl --without-cares --without-pam --without-ldap --without-systemd && make -s pgbouncer) >> "$out/build.log" 2>&1
fi
# A free port on the loopback for PgBouncer.
port=$(python3 -I -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
printf '"%s" "%s"\n' "$PGUSER" "$PGPASSWORD" > "$out/users.txt"
cat > "$out/pgbouncer.ini" <<CONFIG
[databases]
$PGDATABASE = host=$PGHOST port=$PGPORT dbname=$PGDATABASE

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = $port
unix_socket_dir =
auth_type = plain
auth_file = $out/users.txt
pool_mode = transaction
max_prepared_statements = 100
default_pool_size = 2
logfile = $out/pgbouncer.log
CONFIG
"$out/pgbouncer-$version/pgbouncer" "$out/pgbouncer.ini" > /dev/null 2>&1 &
pgbouncer=$!
trap 'kill $pgbouncer' EXIT
i=0
until "$bin/pg_isready" -q -h 127.0.0.1 -p "$port"; do
    i=$((i + 1))
    [ $i -lt 60 ] || { echo "PgBouncer did not start" >&2; exit 1; }
    sleep 1
done
export PGHOST=127.0.0.1 PGPORT="$port"
# 1. The psql scenario.
"$bin/psql" -X -q -f "$dir/../psql/scenario.sql"
# 2. pgbench with prepared statements on two connections. The server makes the rows of the tables, so the trace does not hold them.
"$bin/pgbench" -q -i -I dtGp -s 1
"$bin/pgbench" -n -c 2 -j 1 -t 20 -M prepared > /dev/null
"$bin/pgbench" -q -i -I d
