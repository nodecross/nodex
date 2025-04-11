FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo clean && cargo build --release

FROM ubuntu:latest
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/mock_studio /usr/local/bin/

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 8020
ENV PORT=8020

CMD ["mock_studio"]
