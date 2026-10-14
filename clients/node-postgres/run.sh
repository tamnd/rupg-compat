#!/bin/sh
# The trace scenario for node-postgres. `rupg-compat record node-postgres` runs this script through the proxy.
# With the argument gate, it runs the gate program in place of the scenario, and `rupg-compat gate` runs it.
# It installs the pinned version of package.json and package-lock.json with the Node.js of node.sh, under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../node.sh"
node_client "$dir" node-postgres
node "${1:-scenario}.js" "$dir/../gate-queries.sql"
