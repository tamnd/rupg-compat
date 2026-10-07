#!/bin/sh
# The trace scenario for psql. `rupg-compat record psql` runs this script through the proxy.
# The client is the psql of the oracle build, so its version is the pin of the oracle.
set -eu
dir=$(dirname "$0")
"${RUPG_COMPAT_BIN:?run this script with rupg-compat record psql}/psql" -X -q -f "$dir/scenario.sql"
