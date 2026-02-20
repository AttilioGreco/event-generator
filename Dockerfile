FROM rust:1.84-slim AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

FROM alpine:3.21

RUN apk add --no-cache libgcc

COPY --from=builder /build/target/release/event-generator /usr/local/bin/event-generator

WORKDIR /app

ENTRYPOINT ["event-generator"]
