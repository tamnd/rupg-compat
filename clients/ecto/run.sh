#!/bin/sh
# The trace scenario for Ecto. `rupg-compat record ecto` runs this script through the proxy.
# elixir.sh gets the pinned Erlang/OTP and Elixir under the work directory, and Mix gets the packages of mix.lock there.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../elixir.sh"
elixir_client "$dir" ecto
mix run --no-compile scenario.exs
