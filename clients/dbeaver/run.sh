#!/bin/sh
# The trace scenario for DBeaver. `rupg-compat record dbeaver` runs this script through the proxy.
# DBeaver has no headless mode, so the scenario sends the queries of the PostgreSQL plugin of DBeaver 26.2.3 through pgjdbc 42.7.13, the same pgjdbc pin as clients/pgjdbc/pom.xml.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the driver of pom.xml. The scenario makes the schema compat_dbeaver with the psql of the oracle build and drops it at the end.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record dbeaver}"
. "$dir/../java.sh"
java_client "$dir" dbeaver
maven
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -f "$dir/schema.sql"
java -cp "$(cat cp.txt)" Scenario.java
"$bin/psql" -X -q -v ON_ERROR_STOP=1 -c 'DROP SCHEMA compat_dbeaver CASCADE'
