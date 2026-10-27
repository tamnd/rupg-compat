#!/bin/sh
# The trace scenario for Beekeeper Studio. `rupg-compat record beekeeper` runs this script through the proxy.
# Beekeeper Studio is an Electron app and has no headless mode. The scenario sends the queries of its PostgreSQL client at v6.1.6, in the order of the app, through node-postgres 8.18.0, the driver and version that Beekeeper Studio 6.1.6 pins.
# It installs the pinned version of package.json and package-lock.json with the Node.js of node.sh, under the work directory. It makes the schema compat_beekeeper with the psql of the oracle build and drops it at the end.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record beekeeper}"
. "$dir/../node.sh"
node_client "$dir" beekeeper
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
node scenario.js
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP SCHEMA compat_beekeeper CASCADE'
