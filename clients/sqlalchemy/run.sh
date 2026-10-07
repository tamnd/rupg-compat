#!/bin/sh
# The trace scenario for sqlalchemy. `rupg-compat record sqlalchemy` runs this script through the proxy.
# It installs the pinned versions in a virtual environment under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
env="${RUPG_COMPAT_WORK:-work}/clients/sqlalchemy-2.1.3-alembic-1.20.0"
if [ ! -x "$env/bin/python" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet SQLAlchemy==2.1.3 alembic==1.20.0 psycopg==3.3.6 psycopg-binary==3.3.6
fi
"$env/bin/python" "$dir/scenario.py"
