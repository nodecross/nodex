#
# Copyright 2024 nodex
#
# All Rights Reserved.

name "nodex-agent"
maintainer "CollaboGate Japan"
homepage "https://docs.nodecross.io/"

build_version Omnibus::BuildVersion.semver
build_iteration 1

# Defaults to C:/nodex-agent on Windows
# and /opt/nodex-agent on all other platforms
install_dir "#{default_root}/#{name}"

# Creates required build directories
dependency "preparation"
dependency "init-scripts"
dependency "build-nodex-agent"

exclude "**/.git"
exclude "**/bundler/git"

# handle distribution by environment variable
if ENV['TARGET_PLATFORM'] == 'ubuntu'
  package_scripts_path "#{Omnibus::Config.project_root}/package-scripts/nodex-agent-deb"
end

package :deb do
  compression_level 5
  compression_type :xz
end
