# syntax = docker/dockerfile:1.2.1

FROM rust:latest as builder
USER root

WORKDIR /rust-src
COPY . /rust-src

RUN apt update && apt upgrade -y && apt install -y gcc-x86-64-linux-gnu gcc-aarch64-linux-gnu

RUN rustup target add "$(uname -m)-unknown-linux-musl"

RUN --mount=type=cache,target=/rust-src/target \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --target "$(uname -m)-unknown-linux-musl" -p dockerfile-plus

RUN --mount=type=cache,target=/rust-src/target \
    cp "/rust-src/target/$(uname -m)-unknown-linux-musl/release/dockerfile-plus" /usr/local/bin/dockerfile-plus

FROM docker/dockerfile:1.2.1
COPY --from=builder /usr/local/bin/dockerfile-plus /usr/local/bin/dockerfile-plus
ENTRYPOINT ["/usr/local/bin/dockerfile-plus"]
