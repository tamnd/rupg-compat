# The Ruby of the Ruby clients. Their run.sh sources this file.
# Server1 has no headers of OpenSSL and libyaml, and the prebuilt Ruby archives have a fixed prefix. So this file builds OpenSSL, libyaml and Ruby from their source archives into the work directory, and it checks the SHA-256 of each archive. Then ruby_client copies the files of a client to the work directory and installs its gems with Bundler from Gemfile.lock.
ruby_version=4.0.7
ruby_sha256=47ef59413f7a4587ba6a6b78b14036eb5e36eec2ec0b90964801e88d56a3d375
openssl_version=3.6.5
openssl_sha256=a2157c2830efdec3788939b00c9b0638306d3f0bbb76dc4832ee503bb397df98
yaml_version=0.2.5
yaml_sha256=c642ae9b75fee120b2d96c712538bd2cf283228d2337df2cf2988e3c02678ef4
ruby_dir="${RUPG_COMPAT_WORK:-work}/clients/ruby-$ruby_version"
if [ ! -x "$ruby_dir/bin/ruby" ]; then
    mkdir -p "$ruby_dir/src"
    ruby_dir=$(cd "$ruby_dir" && pwd)
    # ruby_get FILE URL SHA256: downloads an archive, checks it and unpacks it.
    ruby_get() {
        curl -sfL -o "$ruby_dir/src/$1" "$2"
        echo "$3  $ruby_dir/src/$1" | sha256sum -c --quiet
        tar xf "$ruby_dir/src/$1" -C "$ruby_dir/src"
    }
    ruby_get openssl.tar.gz "https://github.com/openssl/openssl/releases/download/openssl-$openssl_version/openssl-$openssl_version.tar.gz" "$openssl_sha256"
    ruby_get yaml.tar.gz "https://github.com/yaml/libyaml/releases/download/$yaml_version/yaml-$yaml_version.tar.gz" "$yaml_sha256"
    ruby_get ruby.tar.xz "https://cache.ruby-lang.org/pub/ruby/${ruby_version%.*}/ruby-$ruby_version.tar.xz" "$ruby_sha256"
    (cd "$ruby_dir/src/openssl-$openssl_version" && ./Configure --prefix="$ruby_dir" --libdir=lib shared no-tests no-docs >/dev/null && make -s -j4 >/dev/null && make -s install_sw >/dev/null)
    (cd "$ruby_dir/src/yaml-$yaml_version" && ./configure -q --prefix="$ruby_dir" --disable-static && make -s >/dev/null && make -s install >/dev/null)
    (cd "$ruby_dir/src/ruby-$ruby_version" && ./configure -q --prefix="$ruby_dir" --with-openssl-dir="$ruby_dir" --with-libyaml-dir="$ruby_dir" --disable-install-doc LDFLAGS="-Wl,-rpath,$ruby_dir/lib" && make -s -j4 >/dev/null && make -s install >/dev/null)
    rm -rf "$ruby_dir/src"
fi
ruby_dir=$(cd "$ruby_dir" && pwd)
PATH="$ruby_dir/bin:$PATH"
export PATH BUNDLE_USER_HOME="$ruby_dir/bundle" BUNDLE_FROZEN=true

# ruby_client DIR NAME: copies the client files in DIR to the work directory, installs the gems of Gemfile.lock there and changes to that directory.
ruby_client() {
    work="${RUPG_COMPAT_WORK:-work}/clients/$2"
    mkdir -p "$work"
    cp -R "$1/." "$work/"
    cd "$work"
    BUNDLE_PATH="$work/gems"
    export BUNDLE_PATH
    bundle install --quiet
}
