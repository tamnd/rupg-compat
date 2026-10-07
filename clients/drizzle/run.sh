#!/bin/sh
# The trace scenario for Drizzle. `rupg-compat record drizzle` runs this script through the proxy.
# It installs the pinned versions of package.json and package-lock.json with the Node.js of node.sh, under the work directory.
# Drizzle Kit makes the schema with `push`, the scenario uses it, a second `push` finds no change, `pull` reads the schema back, and psql drops it.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record drizzle}"
. "$dir/../node.sh"
node_client "$dir" drizzle
npx drizzle-kit push --config drizzle.config.js --force
node scenario.js
npx drizzle-kit push --config drizzle.config.js --force
rm -rf drizzle
npx drizzle-kit pull --config drizzle.config.js
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_orders, compat_items; DROP TYPE compat_mood'
