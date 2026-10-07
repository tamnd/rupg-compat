#!/bin/sh
# The trace scenario for node-postgres. `rupg-compat record node-postgres` runs this script through the proxy.
# It installs the pinned version of package.json and package-lock.json with the Node.js of node.sh, under the work directory.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../node.sh"
node_client "$dir" node-postgres
node scenario.js
