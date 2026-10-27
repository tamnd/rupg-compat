#!/bin/sh
# The trace scenario for psql. `rupg-compat record psql` runs this script through the proxy.
# With the argument gate, it runs the gate program in place of the scenario, and `rupg-compat gate` runs it.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# The client is the psql of the oracle build, so its version is the pin of the oracle.
set -eu
dir=$(dirname "$0")
"${RUPG_COMPAT_BIN:?run this script with rupg-compat record psql}/psql" -X -q -f "$dir/${1:-scenario}.sql"
