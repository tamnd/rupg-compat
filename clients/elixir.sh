# The Erlang/OTP and Elixir of the Elixir clients. Their run.sh sources this file.
# It gets the pinned OTP build of builds.hex.pm for Ubuntu 24.04 and the pinned Elixir release into the work directory, and it checks the SHA-256 of each archive. The versions are the versions of .tool-versions of Supavisor. Then elixir_client copies the files of a client to the work directory and gets its packages from mix.lock.
otp_version=27.3.4.16
otp_sha256=b929b15244fa9c44e610012b0e8582bd5b43819f186107ef3f1f1c2cbec3b351
elixir_version=1.18.5
elixir_sha256=56a065fa2bc609f49a4a3950d509a6a9aa9a827e8610b717e6cfca74a9090580
# Mix checks the signature of the Hex archive.
hex_version=2.5.1
beam_dir="${RUPG_COMPAT_WORK:-work}/clients/elixir-$elixir_version-otp-$otp_version"
if [ ! -x "$beam_dir/elixir/bin/elixir" ]; then
    mkdir -p "$beam_dir/otp" "$beam_dir/elixir"
    beam_dir=$(cd "$beam_dir" && pwd)
    curl -sfL -o "$beam_dir/otp.tar.gz" "https://builds.hex.pm/builds/otp/amd64/ubuntu-24.04/OTP-$otp_version.tar.gz"
    echo "$otp_sha256  $beam_dir/otp.tar.gz" | sha256sum -c --quiet
    tar xzf "$beam_dir/otp.tar.gz" -C "$beam_dir/otp" --strip-components 1
    (cd "$beam_dir/otp" && ./Install -minimal "$beam_dir/otp" >/dev/null)
    curl -sfL -o "$beam_dir/elixir.zip" "https://github.com/elixir-lang/elixir/releases/download/v$elixir_version/elixir-otp-27.zip"
    echo "$elixir_sha256  $beam_dir/elixir.zip" | sha256sum -c --quiet
    unzip -q "$beam_dir/elixir.zip" -d "$beam_dir/elixir"
    rm "$beam_dir/otp.tar.gz" "$beam_dir/elixir.zip"
fi
beam_dir=$(cd "$beam_dir" && pwd)
PATH="$beam_dir/elixir/bin:$beam_dir/otp/bin:$PATH"
export PATH MIX_HOME="$beam_dir/mix" HEX_HOME="$beam_dir/hex" ERL_AFLAGS="-kernel shell_history disabled"

# elixir_client DIR NAME: copies the client files in DIR to the work directory, gets the packages of mix.lock, compiles them and changes to that directory.
elixir_client() {
    work="${RUPG_COMPAT_WORK:-work}/clients/$2"
    mkdir -p "$work"
    cp -R "$1/." "$work/"
    cd "$work"
    mix local.hex "$hex_version" --force --if-missing >/dev/null
    mix deps.get --check-locked >/dev/null
    mix compile >/dev/null
}
