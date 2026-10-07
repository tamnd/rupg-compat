#!/bin/sh
# The trace scenario for pgjdbc. `rupg-compat record pgjdbc` runs this script through the proxy.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the driver of pom.xml.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" pgjdbc
maven
java -cp "$(cat cp.txt)" Scenario.java
