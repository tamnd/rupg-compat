#!/bin/sh
# The trace scenario for asyncpg. `rupg-compat record asyncpg` runs this script through the proxy.
# It installs the pinned version in a virtual environment under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
env="${RUPG_COMPAT_WORK:-work}/clients/asyncpg-0.32.0"
if [ ! -x "$env/bin/python" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet asyncpg==0.32.0
fi
"$env/bin/python" "$dir/scenario.py"
