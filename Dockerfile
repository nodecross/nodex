# FROM messense/rust-musl-cross:x86_64-musl as builder
# WORKDIR /home/rust/src
# RUN rustup target add x86_64-unknown-linux-musl
# COPY Cargo.toml Cargo.toml
# RUN mkdir src/
# RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
# RUN cargo build --release
# RUN rm -f target/release/deps/app*
# 
# COPY . .
# RUN cargo build --release

FROM alpine:3.16.2
# COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/unid-agent /usr/local/bin/unid-agent
RUN apk add bash
RUN apk add curl
COPY target/x86_64-unknown-linux-musl/release/unid-agent /usr/local/bin/unid-agent
CMD ["bash"]