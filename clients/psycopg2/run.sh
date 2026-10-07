#!/bin/sh
# The trace scenario for psycopg2. `rupg-compat record psycopg2` runs this script through the proxy.
# It installs the pinned version in a virtual environment under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
env="${RUPG_COMPAT_WORK:-work}/clients/psycopg2-2.9.13"
if [ ! -x "$env/bin/python" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet psycopg2-binary==2.9.13
fi
"$env/bin/python" "$dir/scenario.py"
