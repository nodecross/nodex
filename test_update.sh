#!/bin/bash
# clean up
pkill -9 nodex-agent
rm /dev/shm/nodex_runtime_info

sed -i 's/^version.*=.*".*\..*\..*"/version = "3.4.1"/' Cargo.toml
mkdir -p /tmp/nodex-deploy/
cargo build --release
pushd target/release/
zip -r /tmp/nodex-deploy/nodex-agent.zip nodex-agent
popd
sed -i 's/^version.*=.*".*\..*\..*"/version = "3.4.0"/' Cargo.toml
cargo build --release
pushd target/release/
./nodex-agent controller &
popd

sleep 1
python3 examples/python/src/get_version_loop.py &
child_pid=$!
sleep 1
python3 examples/python/src/update_version.py
sleep 5
kill $child_pid
