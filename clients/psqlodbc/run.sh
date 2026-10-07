#!/bin/sh
# The trace scenario for psqlODBC. `rupg-compat record psqlodbc` runs this script through the proxy.
# It builds the pinned psqlODBC with the libpq of the oracle build and the unixODBC driver manager, from their source archives under the work directory, and checks their SHA-256. psqlODBC has no source archive for Linux, so the build makes its configure script from the tag with autoconf, automake and libtool. The scenario is a C program that runs the 10 steps of the driver scenario through the driver manager.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
bin="${RUPG_COMPAT_BIN:?run this script with rupg-compat record psqlodbc}"
version=18.00.0004
sha256=af12354a5960846f5578e168b456cde3c21a11c4788277663bf9266b1de3adda
unixodbc=2.3.14
unixodbc_sha256=4e2814de3e01fc30b0b9f75e83bb5aba91ab0384ee951286504bb70205524771
# The GNU archives have good signatures from the GNU keyring.
autoconf=2.73
autoconf_sha256=9fd672b1c8425fac2fa67fa0477b990987268b90ff36d5f016dae57be0d6b52e
automake=1.19
automake_sha256=e3e2c2e3abf37898138db5b6c1d1dc35c9160c5978be7947d2c741705251d445
libtool=2.6.2
libtool_sha256=2ef1067c16c97db930fd740cc9bc3d3ba9a583804ae5ac42cc3e8719e49e191e
out="${RUPG_COMPAT_WORK:-work}/clients/psqlodbc-$version"
mkdir -p "$out"
out=$(cd "$out" && pwd)
driver="$out/odbc/lib/psqlodbcw.so"
if [ ! -f "$driver" ]; then
    # get FILE URL SHA256: downloads an archive, checks it and unpacks it.
    get() {
        curl -sfL -o "$out/$1" "$2"
        echo "$3  $out/$1" | sha256sum -c --quiet
        tar xf "$out/$1" -C "$out"
    }
    get autoconf.tar.xz "https://ftp.gnu.org/gnu/autoconf/autoconf-$autoconf.tar.xz" "$autoconf_sha256"
    get automake.tar.xz "https://ftp.gnu.org/gnu/automake/automake-$automake.tar.xz" "$automake_sha256"
    get libtool.tar.xz "https://ftp.gnu.org/gnu/libtool/libtool-$libtool.tar.xz" "$libtool_sha256"
    get unixodbc.tar.gz "https://github.com/lurcher/unixODBC/releases/download/v$unixodbc/unixODBC-$unixodbc.tar.gz" "$unixodbc_sha256"
    get psqlodbc.tar.gz "https://github.com/postgresql-interfaces/psqlodbc/archive/refs/tags/REL-$(echo "$version" | tr . _).tar.gz" "$sha256"
    PATH="$out/tools/bin:$bin:$PATH"
    for p in "autoconf-$autoconf" "automake-$automake" "libtool-$libtool"; do
        (cd "$out/$p" && ./configure -q --prefix="$out/tools" && make -s && make -s install) >> "$out/build.log" 2>&1
    done
    (cd "$out/unixODBC-$unixodbc" && ./configure -q --prefix="$out/odbc" --disable-gui --disable-readline && make -s && make -s install) >> "$out/build.log" 2>&1
    (cd "$out/psqlodbc-REL-$(echo "$version" | tr . _)" && autoreconf -fi && ./configure -q --prefix="$out/odbc" --with-unixodbc="$out/odbc/bin/odbc_config" LDFLAGS="-Wl,-rpath,$("$bin/pg_config" --libdir)" && make -s && make -s install) >> "$out/build.log" 2>&1
fi
cc -std=c11 -Wall -Wextra -Werror -O2 -I"$out/odbc/include" -o "$out/scenario" "$dir/scenario.c" -L"$out/odbc/lib" -Wl,-rpath,"$out/odbc/lib" -lodbc
# The driver manager reads its settings from the work directory and finds the driver by its path.
ODBCSYSINI="$out" ODBCINI="$out/odbc.ini" "$out/scenario" "$driver"
