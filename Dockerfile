FROM oven/bun:1-alpine AS dashboard-builder
ENV NODE_ENV=production

WORKDIR /dashboard

COPY dashboard/package.json dashboard/bun.lock* ./
RUN bun install

COPY dashboard/ ./
RUN bun run build

FROM rust:1.93-alpine3.23 AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev && cargo install cargo-chef

FROM chef AS planner
COPY . /app
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder
COPY . /app

COPY --from=dashboard-builder /dashboard/build /app/dashboard/build

COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release

FROM alpine:3.23

COPY --from=builder /app/target/release/event-generator /usr/local/bin/event-generator

WORKDIR /app

# ENTRYPOINT ["event-generator"]

CMD [ "event-generator" ]

