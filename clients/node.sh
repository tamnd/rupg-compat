# The Node.js of the Node clients. Their run.sh sources this file.
# It gets the pinned release into the work directory, checks its SHA-256 and puts it first on PATH. Then node_client copies the files of a client to the work directory and installs its packages with `npm ci` from package-lock.json.
node_version=24.21.0
node_sha256=fd8e59d5a511510f6a298afb548f18c7d2b1be404d8b4a27d94fbe49f56cb2d6
node_dir="${RUPG_COMPAT_WORK:-work}/clients/node-v$node_version"
if [ ! -x "$node_dir/bin/node" ]; then
    mkdir -p "$node_dir"
    curl -sfL -o "$node_dir.tar.xz" "https://nodejs.org/dist/v$node_version/node-v$node_version-linux-x64.tar.xz"
    echo "$node_sha256  $node_dir.tar.xz" | sha256sum -c --quiet
    tar xJf "$node_dir.tar.xz" -C "$node_dir" --strip-components 1
    rm "$node_dir.tar.xz"
fi
PATH="$node_dir/bin:$PATH"
export PATH npm_config_cache="$node_dir/cache" npm_config_update_notifier=false npm_config_fund=false npm_config_audit=false

# node_client DIR NAME: copies the client files in DIR to the work directory, installs the packages and changes to that directory.
node_client() {
    work="${RUPG_COMPAT_WORK:-work}/clients/$2"
    mkdir -p "$work"
    cp -R "$1/." "$work/"
    cd "$work"
    [ -d node_modules ] || npm ci --silent
}
