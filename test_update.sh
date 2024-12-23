#!/bin/bash
# clean up
pkill -9 nodex-agent
rm /dev/shm/nodex_runtime_info
rm ~/.nodex/run/nodex.sock
rm  ~/.nodex/run/meta_nodex.sock

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
bash -c "while true; do curl -H 'Content-Type:application/json' --unix-socket ~/.nodex/run/nodex.sock localhost/internal/version/get; done" &
child_pid=$!
sleep 1
curl -v -X POST -H 'Content-Type:application/json' -d '{"message":{"binary_url":"http://localhost:9000/nodex-agent.zip"}}' --unix-socket ~/.nodex/run/nodex.sock http://localhost/internal/version/update
sleep 5
kill $child_pid
