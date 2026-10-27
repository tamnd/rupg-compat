#!/bin/sh
# The trace scenario for postgres-js. `rupg-compat record postgres-js` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# It installs the pinned version of package.json and package-lock.json with the Node.js of node.sh, under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../node.sh"
node_client "$dir" postgres-js
node "${1:-scenario}.js"
