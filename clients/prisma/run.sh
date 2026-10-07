#!/bin/sh
# The trace scenario for Prisma. `rupg-compat record prisma` runs this script through the proxy.
# It installs the pinned versions of package.json and package-lock.json with the Node.js of node.sh, under the work directory. The install gets the Prisma schema engine for this platform.
# `prisma/migrations` holds the output of `prisma migrate diff --from-empty --to-schema prisma/schema.prisma --script`. The Prisma CLI applies the migration with `migrate deploy`, the scenario uses the tables with Prisma Client, then `migrate status` and `db pull` read the schema, and psql drops it.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record prisma}"
. "$dir/../node.sh"
# No telemetry and no update notice.
export CHECKPOINT_DISABLE=1 PRISMA_HIDE_UPDATE_MESSAGE=1
node_client "$dir" prisma
npx prisma generate --no-hints
npx prisma migrate deploy
node scenario.js
npx prisma migrate status
npx prisma db pull --print > /dev/null
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_orders, compat_items, _prisma_migrations; DROP TYPE compat_mood'
