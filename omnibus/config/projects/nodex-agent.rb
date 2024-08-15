#
# Copyright 2024 nodex
#
# All Rights Reserved.

name "nodex-agent"
maintainer "CHANGE ME"
homepage "https://CHANGE-ME.com"

# Defaults to C:/nodex-agent on Windows
# and /opt/nodex-agent on all other platforms
install_dir "#{default_root}/#{name}"

build_version Omnibus::BuildVersion.semver
build_iteration 1

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
