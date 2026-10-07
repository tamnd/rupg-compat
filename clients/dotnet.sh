# The .NET of the .NET clients. Their run.sh sources this file.
# It gets the pinned .NET SDK into the work directory, checks its SHA-512 and puts it first on PATH. dotnet_client copies the files of a client to the work directory and restores its NuGet packages from packages.lock.json in locked mode, into a package folder in the work directory.
dotnet_version=10.0.401
dotnet_sha512=51c8b999af9e8dd9998c9edc5944e19a90788862068acd38694e098889054ce8c23d4f0c5cccfa16bf187d044562359e5ee69a9f8ad0bbe913ba90311fbce25b
dotnet_work="${RUPG_COMPAT_WORK:-work}/clients"
dotnet_dir="$dotnet_work/dotnet-sdk-$dotnet_version"
if [ ! -x "$dotnet_dir/dotnet" ]; then
    mkdir -p "$dotnet_dir"
    curl -sfL -o "$dotnet_dir.tar.gz" "https://builds.dotnet.microsoft.com/dotnet/Sdk/$dotnet_version/dotnet-sdk-$dotnet_version-linux-x64.tar.gz"
    echo "$dotnet_sha512  $dotnet_dir.tar.gz" | sha512sum -c --quiet
    tar xzf "$dotnet_dir.tar.gz" -C "$dotnet_dir"
    rm "$dotnet_dir.tar.gz"
fi
# The SDK sends no usage data, prints no welcome text and keeps its files in the work directory.
DOTNET_ROOT="$dotnet_dir"
PATH="$dotnet_dir:$PATH"
DOTNET_CLI_TELEMETRY_OPTOUT=1
DOTNET_NOLOGO=1
DOTNET_SKIP_FIRST_TIME_EXPERIENCE=1
DOTNET_CLI_HOME="$dotnet_work/dotnet-home"
NUGET_PACKAGES="$dotnet_work/nuget"
export DOTNET_ROOT PATH DOTNET_CLI_TELEMETRY_OPTOUT DOTNET_NOLOGO DOTNET_SKIP_FIRST_TIME_EXPERIENCE DOTNET_CLI_HOME NUGET_PACKAGES

# dotnet_client DIR NAME: copies the client files in DIR to the work directory, restores the packages, builds the project and changes to that directory.
dotnet_client() {
    work="$dotnet_work/$2"
    mkdir -p "$work"
    cp -R "$1/." "$work/"
    cd "$work"
    dotnet restore --locked-mode -v quiet
    dotnet build --no-restore -c Release -v quiet -nologo
}
