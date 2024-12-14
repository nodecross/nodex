#!/bin/bash
# clean up
pkill -9 nodex-agent
rm /dev/shm/runtime_info

sed -i 's/^version.*=.*".*\..*\..*"/version = "3.4.1"/' Cargo.toml
cargo build --release
pushd target/release/
zip -r nodex-agent.zip nodex-agent
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
sleep 4
kill $child_pid
