#!/bin/sh
# The trace scenario for Flyway. `rupg-compat record flyway` runs this script through the proxy.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the Flyway command line of pom.xml.
# Flyway applies two versioned migrations and a repeatable one, shows the state, validates the checksums and cleans the schema.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" flyway
maven
# Flyway sends no usage data and does not look for a new release.
export REDGATE_DISABLE_TELEMETRY=true
flyway() {
    java -cp "$(cat cp.txt)" org.flywaydb.commandline.Main -url="jdbc:postgresql://$PGHOST:$PGPORT/$PGDATABASE?sslmode=disable" -user="$PGUSER" -password="$PGPASSWORD" -locations=filesystem:sql -cleanDisabled=false -skipCheckForUpdate "$@"
}
flyway migrate
flyway info
flyway validate
flyway clean
