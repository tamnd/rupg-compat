#!/bin/sh
# The trace scenario for Active Record, the ORM of Rails. `rupg-compat record activerecord` runs this script through the proxy.
# ruby.sh builds the pinned Ruby under the work directory, and Bundler installs the gems of Gemfile.lock there.
set -eu
dir=$(cd "$(dirname "$0")" && pwd)
. "$dir/../ruby.sh"
ruby_client "$dir" activerecord
bundle exec ruby scenario.rb
