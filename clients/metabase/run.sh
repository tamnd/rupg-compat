#!/bin/sh
# The trace scenario for Metabase. `rupg-compat record metabase` runs this script through the proxy.
# It gets the pinned Metabase JAR under the work directory and checks its SHA-256. Metabase publishes no checksum, so the pin is the SHA-256 of the JAR of the release. Metabase runs on the JDK of java.sh, with its own data in an H2 file in the work directory. The scenario makes two tables with the psql of the oracle build, adds the database to Metabase, waits for the sync and sends the requests of the Metabase web pages to its HTTP API.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record metabase}"
version=0.64.1
sha256=ffe600f532875b4d3c738effbc0f42f8a1ad7f8255cb875569f73b36645ef11d
. "$dir/../java.sh"
out="$java_work/metabase-$version"
mkdir -p "$out"
if ! echo "$sha256  $out/metabase.jar" | sha256sum -c --quiet 2>/dev/null; then
    curl -sfL -o "$out/metabase.jar" "https://downloads.metabase.com/v$version.x/metabase.jar"
    echo "$sha256  $out/metabase.jar" | sha256sum -c --quiet
fi
rm -rf "$out/data"
mkdir -p "$out/data"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
python3 -I "$dir/scenario.py" "$out"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_orders, compat_items'
