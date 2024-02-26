## E2E Test with docker containers

### on apple silicon mac

binary build command

```shell
CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target x86_64-unknown-linux-gnu
```

## Run E2E test

```shell
cp target/x86_64-unknown-linux-gnu/release/nodex-agent test_resource/

docker compose -f test_resource/compose.yaml --profile e2e up -d

docker compose -f test_resource/compose.yaml --profile e2e run e2e_runner cargo test

# check logs of agent
docker compose -f test_resource/compose.yaml --profile e2e logs e2e_agent
```
