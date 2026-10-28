#!/bin/bash
# The nightly run of the page of spec/05 section 5.8: each suite on the oracle and on rupg, then the report.
# Usage: .github/nightly.sh RUPG COMMIT OUT
# RUPG is the rupg binary and COMMIT is its commit. The report goes to OUT/report.md.
# Each server runs in a new cgroup v2 for each suite, so that memory.peak is the peak of that suite also on a kernel before Linux 6.12. The script makes the cgroups with sudo and runs the servers as the current user.
# The exit status is 0 when each suite ran and the report passed its ratchet. The page is written in both cases.
set -u
rupg=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
commit=$2
out=$3
rc="$PWD/target/release/rupg-compat"
work=${RUPG_COMPAT_WORK:?set RUPG_COMPAT_WORK}
logs="$work/nightly"
top=/sys/fs/cgroup/rupg-compat
port=5498
status=0
mkdir -p "$logs" "$out"
sudo mkdir -p "$top"
echo "+cpu +memory" | sudo tee "$top/cgroup.subtree_control" > /dev/null

# cgroup NAME: makes a new cgroup and prints its directory.
cgroup() {
    cg="$top/$1"
    if [ -d "$cg" ]; then sudo rmdir "$cg"; fi
    sudo mkdir "$cg"
    sudo chown "$(id -u)" "$cg/memory.peak"
    echo "$cg"
}

# incg CGROUP CMD...: runs CMD as the current user in CGROUP.
incg() {
    cg=$1
    shift
    sudo sh -c 'cg=$1 user=$2; shift 2; echo $$ > "$cg/cgroup.procs" && exec runuser -u "$user" -- "$@"' \
        sh "$cg" "$(id -un)" env PATH="$PATH" HOME="$HOME" RUPG_COMPAT_WORK="$work" "$@"
}

# stop_rupg: stops each process in the cgroup of rupg, then waits until the port is free.
stop_rupg() {
    if [ -z "${rupg_cg:-}" ]; then return 0; fi
    sudo sh -c 'kill $(cat "$1/cgroup.procs") 2> /dev/null' sh "$rupg_cg"
    i=0
    while [ -s "$rupg_cg/cgroup.procs" ] && [ $i -lt 100 ]; do
        sleep 0.2
        i=$((i + 1))
    done
    if [ -s "$rupg_cg/cgroup.procs" ]; then echo 1 | sudo tee "$rupg_cg/cgroup.kill" > /dev/null; fi
    wait 2> /dev/null
    rupg_cg=
}

# port_free: true when nothing listens on the port of rupg.
port_free() {
    ! (echo > /dev/tcp/127.0.0.1/$port) 2> /dev/null
}

# fresh_rupg CGROUP: starts rupg with a new database in CGROUP.
fresh_rupg() {
    stop_rupg
    if ! port_free; then
        echo "port $port is in use, so rupg cannot start"
        return 1
    fi
    rm -rf "$work/rupg-conf"
    "$rc" oracle conf "$work/rupg-conf" --port $port --compat-version 19 > /dev/null || return 1
    echo rupg-compat > "$work/rupg-pw"
    rupg_cg=$1
    incg "$1" "$rupg" serve -D "$work/rupg-conf" --pwfile "$work/rupg-pw" >> "$logs/rupg.log" 2>&1 &
    i=0
    while [ $i -lt 300 ]; do
        if ! port_free; then return 0; fi
        sleep 0.2
        i=$((i + 1))
    done
    echo "rupg did not start, see $logs/rupg.log"
    return 1
}

# fresh_oracle CGROUP: starts the oracle in CGROUP.
fresh_oracle() {
    "$rc" oracle stop --compat-version 19 > /dev/null 2>&1
    incg "$1" "$rc" oracle start --compat-version 19 >> "$logs/oracle.log" 2>&1
}

# suite NAME ARGS...: runs a suite on the oracle and on rupg, each in a new cgroup. server_env holds the environment of the run on rupg.
suite() {
    name=$1
    shift
    cg=$(cgroup "oracle-$name")
    if ! fresh_oracle "$cg"; then
        echo "$name: the oracle did not start, see $logs/oracle.log"
        status=1
        return
    fi
    "$rc" "$@" --compat-version 19 --cgroup "$cg" > "$logs/$name-oracle.log" 2>&1 || true
    tail -n 1 "$logs/$name-oracle.log"
    cg=$(cgroup "rupg-$name")
    if ! fresh_rupg "$cg"; then
        status=1
        return
    fi
    # shellcheck disable=SC2086 # server_env is a list of NAME=VALUE words.
    env ${server_env:-} "$rc" "$@" --server 127.0.0.1:$port --compat-version 19 --cgroup "$cg" > "$logs/$name-server.log" 2>&1 || true
    tail -n 1 "$logs/$name-server.log"
}

suite pipeline pipeline
suite catalog catalog
suite catalog-gate catalog --schema gate
suite catalog-corpus catalog-corpus
suite parameters parameters
suite types types
suite dump dump
suite copy copy
suite gate gate
suite connect connect
for t in corpus/traces/*.trace; do
    suite "replay-$(basename "$t" .trace)" replay "$t"
done
suite regress regress
suite sqllogictest sqllogictest select1.test select2.test select3.test select4.test select5.test evidence index/between
# The seed is the date, so that a difference of the night can be found again.
suite generate generate --seed "$(date -u +%Y%m%d)" --count 20000
suite logic logic --seed "$(date -u +%Y%m%d)" --count 5000
suite layout layout --seed "$(date -u +%Y%m%d)" --count 1000
# On rupg, a spec that waits for a lock that rupg does not take would wait 180 s for each step.
server_env=PG_TEST_TIMEOUT_DEFAULT=10 suite isolation isolation
stop_rupg
"$rc" oracle stop --compat-version 19 > /dev/null 2>&1

"$rc" report --compat-version 19 --ratchet ratchet.toml --rupg-commit "$commit" --out "$out" > "$logs/report.log" 2>&1 || status=1
cat "$logs/report.log"
exit $status
