#!/bin/sh
# The trace scenario for Npgsql. `rupg-compat record npgsql` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# dotnet.sh gets the pinned .NET SDK under the work directory, restores the package of packages.lock.json and builds the program.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../dotnet.sh"
# The build writes to the standard output, so its output goes to the standard error, and the standard output has only the output of the program.
dotnet_client "$dir" npgsql >&2
dotnet run --no-build -c Release -- "$@"
