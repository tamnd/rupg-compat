#!/bin/sh
# The trace scenario for pgAdmin 4. `rupg-compat record pgadmin4` runs this script through the proxy.
# It installs the pinned version in a virtual environment under the work directory, with the pins of constraints.txt. pgAdmin runs in desktop mode with a new data directory for each run. The scenario makes a table with the psql of the oracle build, then sends the requests of the pgAdmin web pages to its HTTP API.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record pgadmin4}"
version=9.18
env="${RUPG_COMPAT_WORK:-work}/clients/pgadmin4-$version"
if [ ! -x "$env/bin/pgadmin4" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet -c "$dir/constraints.txt" "pgadmin4==$version"
fi
home="$env/home"
rm -rf "$home"
mkdir -p "$home"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
"$env/bin/python" -I "$dir/scenario.py" "$env" "$home"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_items'
