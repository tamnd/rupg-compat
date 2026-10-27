#!/bin/sh
# The trace scenario for the Ruby pg gem. `rupg-compat record ruby-pg` runs this script through the proxy.
# With the argument connect, it runs the connect program in place of the scenario, and `rupg-compat connect` runs it.
# ruby.sh builds the pinned Ruby under the work directory, and Bundler installs the gem of Gemfile.lock there.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../ruby.sh"
ruby_client "$dir" ruby-pg
bundle exec ruby "${1:-scenario}.rb"
