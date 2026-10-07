#!/bin/sh
# The trace scenario for EF Core with Npgsql. `rupg-compat record efcore` runs this script through the proxy.
# dotnet.sh gets the pinned .NET SDK under the work directory, restores the packages of packages.lock.json and builds the program.
# The program applies the migration in Migrations/ and uses the tables. `dotnet ef dbcontext scaffold` reads the schema of each table back from the database. Then the program reverts the migration.
# Each table has its own scaffold. The column query of the provider sorts by attnum only. With two tables, the order of the rows with the same attnum is not defined, and the trace would depend on it.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../dotnet.sh"
dotnet_client "$dir" efcore
dotnet tool restore -v quiet
dotnet run --no-build -c Release -- up
conn="Host=$PGHOST;Port=$PGPORT;Database=$PGDATABASE;Username=$PGUSER;Password=$PGPASSWORD;SSL Mode=Disable;Pooling=false"
for table in compat_items compat_orders; do
    dotnet ef dbcontext scaffold "$conn" Npgsql.EntityFrameworkCore.PostgreSQL --table "$table" --no-build --configuration Release --output-dir "scaffold/$table" --context "${table}_context" --no-onconfiguring --force
done
dotnet run --no-build -c Release -- down
