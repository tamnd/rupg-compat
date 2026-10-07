#!/bin/sh
# The trace scenario for Hibernate ORM. `rupg-compat record hibernate` runs this script through the proxy.
# java.sh gets the pinned JDK and Maven under the work directory, and Maven gets the libraries of pom.xml.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../java.sh"
java_client "$dir" hibernate
maven
rm -rf classes
javac -d classes -cp "$(cat cp.txt)" Scenario.java
java -cp "classes:$(cat cp.txt)" Scenario
