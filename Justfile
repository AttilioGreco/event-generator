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

release:
    bash scripts/release.sh

release-check:
    @echo "Current version: $$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)"
    @echo "Latest local tag: $$(git describe --tags --abbrev=0 2>/dev/null || echo 'no tags')"
    @echo ""
    @echo "GitHub workflow file: .github/workflows/release.yml"

docker-build:
    docker build -t event-generator .

docker-build-debug:
    docker build --platform linux/amd64 -t event-generator-debug:latest .

docker-run CONFIG="config/example.toml":
    docker run --rm -v "$(pwd)/{{CONFIG}}:/config.toml" event-generator --config /config.toml

docker-run-debug CONFIG="config/example.toml":
    docker run --rm -v "$(pwd)/{{CONFIG}}:/config.toml" event-generator-debug:latest --config /config.toml
