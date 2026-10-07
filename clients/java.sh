# The Java of the Java clients. Their run.sh sources this file.
# It gets the pinned Eclipse Temurin JDK into the work directory, checks its SHA-256 and puts it first on PATH. java_client copies the files of a client to the work directory. maven gets the pinned Apache Maven, checks its SHA-512 and resolves the dependencies of pom.xml into a local repository in the work directory, with strict checksums.
jdk_version=21.0.12.1+1
jdk_sha256=ce79869e1307ed8ee1e2baa86a412b1eb5b75d10a01006d788a6f968bcfaee94
maven_version=3.9.16
maven_sha512=831a8591fe20c8243b1dbe7d71e3244f31d1665b0804b2e825e38cbbe5ce0cafb8338851f90780735568773e0a6cd07bbec107cda0b896b008b861075358b6f6
java_work="${RUPG_COMPAT_WORK:-work}/clients"
jdk_dir="$java_work/jdk-$jdk_version"
if [ ! -x "$jdk_dir/bin/java" ]; then
    mkdir -p "$jdk_dir"
    curl -sfL -o "$jdk_dir.tar.gz" "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-$(echo "$jdk_version" | sed 's/+/%2B/')/OpenJDK21U-jdk_x64_linux_hotspot_$(echo "$jdk_version" | tr + _).tar.gz"
    echo "$jdk_sha256  $jdk_dir.tar.gz" | sha256sum -c --quiet
    tar xzf "$jdk_dir.tar.gz" -C "$jdk_dir" --strip-components 1
    rm "$jdk_dir.tar.gz"
fi
JAVA_HOME="$jdk_dir"
PATH="$jdk_dir/bin:$PATH"
export JAVA_HOME PATH

# java_client DIR NAME: copies the client files in DIR to the work directory and changes to that directory.
java_client() {
    work="$java_work/$2"
    mkdir -p "$work"
    cp -R "$1/." "$work/"
    cd "$work"
}

# maven: gets Maven, resolves the dependencies of pom.xml and writes their class path to cp.txt.
maven() {
    maven_dir="$java_work/apache-maven-$maven_version"
    if [ ! -x "$maven_dir/bin/mvn" ]; then
        mkdir -p "$maven_dir"
        curl -sfL -o "$maven_dir.tar.gz" "https://archive.apache.org/dist/maven/maven-3/$maven_version/binaries/apache-maven-$maven_version-bin.tar.gz"
        echo "$maven_sha512  $maven_dir.tar.gz" | sha512sum -c --quiet
        tar xzf "$maven_dir.tar.gz" -C "$maven_dir" --strip-components 1
        rm "$maven_dir.tar.gz"
    fi
    "$maven_dir/bin/mvn" -q -B -C -Dmaven.repo.local="$java_work/m2" org.apache.maven.plugins:maven-dependency-plugin:3.11.0:build-classpath -Dmdep.outputFile=cp.txt
}
