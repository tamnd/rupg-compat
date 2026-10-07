#!/bin/sh
# The trace scenario for Npgsql. `rupg-compat record npgsql` runs this script through the proxy.
# dotnet.sh gets the pinned .NET SDK under the work directory, restores the package of packages.lock.json and builds the program.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../dotnet.sh"
dotnet_client "$dir" npgsql
dotnet run --no-build -c Release
