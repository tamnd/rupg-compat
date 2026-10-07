#!/bin/sh
# The trace scenario for Postgrex. `rupg-compat record postgrex` runs this script through the proxy.
# elixir.sh gets the pinned Erlang/OTP and Elixir under the work directory, and Mix gets the packages of mix.lock there.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../elixir.sh"
elixir_client "$dir" postgrex
mix run --no-compile scenario.exs
