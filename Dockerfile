FROM rust:1.93-alpine3.23 AS chef
WORKDIR /app
RUN cargo install cargo-chef

# (2) generate recipe file to prepare dependencies build
FROM chef AS planner
COPY . /app
RUN cargo chef prepare --recipe-path recipe.json

# (3) build dependencies
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# (4) build app
FROM chef AS builder
COPY . /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release

FROM alpine:3.23

COPY --from=builder /app/target/release/event-generator /usr/local/bin/event-generator

WORKDIR /app

ENTRYPOINT ["event-generator"]

