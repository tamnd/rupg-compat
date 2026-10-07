#!/bin/sh
# The trace scenario for Diesel CLI. `rupg-compat record diesel` runs this script through the proxy.
# It installs the pinned version under the work directory, with the libpq of the oracle build. A migration tool does not send the same statements as a driver. The scenario sets up the database and runs the migrations, lists them, prints the schema, redoes the last migration and reverts all of them.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record diesel}"
version=2.3.13
out="${RUPG_COMPAT_WORK:-work}/clients/diesel-$version"
lib=$("$bin/pg_config" --libdir)
export LD_LIBRARY_PATH="$lib"
if [ ! -x "$out/bin/diesel" ]; then
    PQ_LIB_DIR="$lib" CARGO_TARGET_DIR="$out/target" cargo install --quiet --locked --root "$out" --version "$version" diesel_cli --no-default-features --features postgres
fi
# Diesel takes a URL, so the scenario builds it from the libpq environment.
export DATABASE_URL="postgres://$PGUSER:$PGPASSWORD@$PGHOST:$PGPORT/$PGDATABASE?sslmode=$PGSSLMODE"
project="$out/project"
rm -rf "$project"
mkdir -p "$project"
cp -R "$dir/migrations" "$project/"
# Diesel looks for diesel.toml or Cargo.toml to find the project. An empty diesel.toml keeps the defaults.
: > "$project/diesel.toml"
cd "$project"
# Diesel finds diesel.toml and migrations/ in the current directory.
diesel() {
    "$out/bin/diesel" "$@"
}
# 1. Set up the database and run all migrations.
diesel setup
# 2. List the migrations.
diesel migration list
# 3. Print the schema, which reads the catalogs.
diesel print-schema
# 4. Redo the last migration.
diesel migration redo
# 5. Revert all migrations.
diesel migration revert --all
