# syntax = docker/dockerfile:1.2.1

FROM clux/muslrust:stable as builder
USER root

WORKDIR /rust-src
COPY . /rust-src

RUN --mount=type=cache,target=/rust-src/target \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cargo/registry \
    ["cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl", "-p", "dockerfile-plus"]

RUN --mount=type=cache,target=/rust-src/target \
    ["cp", "/rust-src/target/x86_64-unknown-linux-musl/release/dockerfile-plus", "/usr/local/bin/dockerfile-plus"]

FROM docker/dockerfile:1.2.1
COPY --from=builder /usr/local/bin/dockerfile-plus /usr/local/bin/dockerfile-plus
ENTRYPOINT ["/usr/local/bin/dockerfile-plus"]