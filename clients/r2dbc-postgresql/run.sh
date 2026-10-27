#!/bin/sh
# The trace scenario for the R2DBC driver for PostgreSQL. `rupg-compat record r2dbc-postgresql` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the driver of pom.xml with its dependencies.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" r2dbc-postgresql
maven
if [ "${1:-}" = connect ]; then
    java -cp "$(cat cp.txt)" Connect.java
else
    java -cp "$(cat cp.txt)" Scenario.java
fi
