#!/bin/sh
# The trace scenario for Superset. `rupg-compat record superset` runs this script through the proxy.
# It installs the pinned version in a virtual environment under the work directory, with the pins of constraints.txt. The scenario makes a table with the psql of the oracle build, then sends the requests of the Superset web pages to its REST API.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record superset}"
env="${RUPG_COMPAT_WORK:-work}/clients/superset-6.1.0"
if [ ! -x "$env/bin/superset" ]; then
    python3 -m venv "$env"
    "$env/bin/pip" install --quiet -c "$dir/constraints.txt" apache-superset==6.1.0 psycopg2-binary==2.9.13
fi
# The metadata of Superset is a new SQLite file for each run.
home="$env/home"
rm -rf "$home"
mkdir -p "$home"
cat > "$home/superset_config.py" <<CONFIG
SECRET_KEY = "compat-scenario-key"
SQLALCHEMY_DATABASE_URI = "sqlite:///$home/superset.db"
WTF_CSRF_ENABLED = False
CONFIG
export SUPERSET_CONFIG_PATH="$home/superset_config.py" SUPERSET_HOME="$home"
"$env/bin/superset" db upgrade > "$home/setup.log" 2>&1
"$env/bin/superset" fab create-admin --username admin --firstname compat --lastname compat --email admin@example.com --password compat >> "$home/setup.log" 2>&1
"$env/bin/superset" init >> "$home/setup.log" 2>&1
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
"$env/bin/python" "$dir/scenario.py"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_sales, compat_regions'
