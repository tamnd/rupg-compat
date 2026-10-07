#!/bin/sh
# The trace scenario for Grafana. `rupg-compat record grafana` runs this script through the proxy.
# It downloads the pinned release under the work directory and checks its SHA-256. The scenario makes a table with the psql of the oracle build, starts Grafana with a provisioned PostgreSQL data source and sends the requests of the Grafana web pages to its HTTP API.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record grafana}"
version=13.2.3
sha256=6107ad27016296aac38e0d7ffa8753ab540b5541ad27e94790f771289d733235
out="${RUPG_COMPAT_WORK:-work}/clients/grafana-$version"
if [ ! -x "$out/grafana-$version/bin/grafana" ]; then
    mkdir -p "$out"
    curl -sfL -o "$out/grafana.tar.gz" "https://dl.grafana.com/oss/release/grafana-$version.linux-amd64.tar.gz"
    echo "$sha256  $out/grafana.tar.gz" | sha256sum -c --quiet
    tar xzf "$out/grafana.tar.gz" -C "$out"
    rm "$out/grafana.tar.gz"
fi
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
python3 -I "$dir/scenario.py" "$out/grafana-$version"
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP TABLE compat_metrics'
