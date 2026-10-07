#!/bin/sh
# The trace scenario for Liquibase. `rupg-compat record liquibase` runs this script through the proxy.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets Liquibase of pom.xml.
# Liquibase applies three change sets, shows the status and the history, takes a snapshot of the schema, rolls back the last change set and drops all objects.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" liquibase
maven
# Liquibase writes the host name and the address of the machine into its lock table. With a hosts file that has only localhost, Java cannot resolve the host name, and Liquibase writes "unknown" in place of both. The JDBC URL uses the address 127.0.0.1, so it needs no name.
printf '127.0.0.1 localhost\n' > hosts
liquibase() {
    java -Djdk.net.hosts.file=hosts -cp "$(cat cp.txt)" liquibase.integration.commandline.LiquibaseCommandLine --url="jdbc:postgresql://$PGHOST:$PGPORT/$PGDATABASE?sslmode=disable" --username="$PGUSER" --password="$PGPASSWORD" --changelog-file=changelog.xml --show-banner=false --analytics-enabled=false "$@"
}
liquibase update
liquibase status
liquibase history
liquibase snapshot > /dev/null
liquibase rollback-count --count=1
liquibase drop-all --force
