#!/bin/sh
# The trace scenario for pgjdbc. `rupg-compat record pgjdbc` runs this script through the proxy.
# With the argument gate, it runs the gate program in place of the scenario, and `rupg-compat gate` runs it.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the driver of pom.xml.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" pgjdbc
maven
if [ "${1:-}" = gate ]; then
    java -cp "$(cat cp.txt)" Gate.java
else
    java -cp "$(cat cp.txt)" Scenario.java
fi
