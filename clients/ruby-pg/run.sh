#!/bin/sh
# The trace scenario for the Ruby pg gem. `rupg-compat record ruby-pg` runs this script through the proxy.
# ruby.sh builds the pinned Ruby under the work directory, and Bundler installs the gem of Gemfile.lock there.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../ruby.sh"
ruby_client "$dir" ruby-pg
bundle exec ruby scenario.rb
