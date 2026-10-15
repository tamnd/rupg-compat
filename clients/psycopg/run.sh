#!/bin/sh
# The trace scenario for psycopg. `rupg-compat record psycopg` runs this script through the proxy.
# With the argument gate, it runs the gate program in place of the scenario, and `rupg-compat gate` runs it.
# It installs the pinned version in a virtual environment under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
env="${RUPG_COMPAT_WORK:-work}/clients/psycopg-3.3.6"
if [ ! -x "$env/bin/python" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet psycopg==3.3.6 psycopg-binary==3.3.6
fi
"$env/bin/python" "$dir/${1:-scenario}.py" "$dir/../gate-queries.sql"
