#!/bin/sh
# The trace scenario for the R2DBC driver for PostgreSQL. `rupg-compat record r2dbc-postgresql` runs this script through the proxy.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the driver of pom.xml with its dependencies.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" r2dbc-postgresql
maven
java -cp "$(cat cp.txt)" Scenario.java
