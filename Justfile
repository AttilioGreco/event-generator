default: build

build:
    cargo build --release

dev:
    cargo build

run CONFIG="config/example.toml":
    cargo run -- --config {{CONFIG}}

test:
    cargo test

lint:
    cargo clippy -- -D warnings

fmt:
    cargo fmt --check

fmt-fix:
    cargo fmt

check: fmt lint test

docker-build:
    docker build -t event-generator .

docker-run CONFIG="config/example.toml":
    docker run --rm -v "$(pwd)/{{CONFIG}}:/config.toml" event-generator --config /config.toml
