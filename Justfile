default: build

# Build dashboard first, then Rust release binary
build: dashboard-build
    cargo build --release

dev:
    cargo build

# Build dashboard and then dev Rust binary
dev-full: dashboard-build
    cargo build

# Run React dashboard dev server (hot reload, proxies to Rust backend)
dashboard-dev:
    cd dashboard && bun run dev

# Build React dashboard for production
dashboard-build:
    cd dashboard && bun run build

# Install dashboard dependencies
dashboard-install:
    cd dashboard && bun install

dev-up:
    docker compose up --build -d

dev-down:
    docker compose down

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

# Full pre-release checklist: fmt, clippy, tests, release build, dashboard build, docker build
pre-release:
    @echo "=== Pre-release checklist ==="
    @echo ""
    @echo "Current version: $$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)"
    @echo "Latest local tag: $$(git describe --tags --abbrev=0 2>/dev/null || echo 'no tags')"
    @echo "Branch: $$(git branch --show-current)"
    @echo "Uncommitted changes: $$(git status --short | wc -l | tr -d ' ')"
    @echo ""
    @echo "[1/6] cargo fmt --check"
    cargo fmt --check
    @echo "  OK"
    @echo "[2/6] cargo clippy -- -D warnings"
    cargo clippy -- -D warnings
    @echo "  OK"
    @echo "[3/6] cargo test"
    cargo test
    @echo "  OK"
    @echo "[4/6] cargo build --release"
    cargo build --release
    @echo "  OK"
    @echo "[5/6] dashboard build"
    cd dashboard && bun run build
    @echo "  OK"
    @echo "[6/6] docker build"
    docker build -t event-generator:pre-release-check .
    @echo "  OK"
    @echo ""
    @echo "=== All checks passed! Ready for release. ==="

release: pre-release
    bash scripts/release.sh

release-info:
    @echo "Current version: $$(grep '^version =' Cargo.toml | head -1 | cut -d'"' -f2)"
    @echo "Latest local tag: $$(git describe --tags --abbrev=0 2>/dev/null || echo 'no tags')"
    @echo ""
    @echo "GitHub workflow file: .github/workflows/release.yml"

docker-build:
    docker build -t event-generator .

docker-build-test TEST_IMAGE="localhost:5000/event-generator:test":
    docker build -t {{TEST_IMAGE}} .

docker-push-test TEST_IMAGE="localhost:5000/event-generator:test":
    just docker-build-test TEST_IMAGE={{TEST_IMAGE}}
    docker push {{TEST_IMAGE}}

docker-build-debug:
    docker build --platform linux/amd64 -t event-generator-debug:latest .

docker-run CONFIG="config/example.toml":
    docker run --rm -v "$PWD/{{CONFIG}}:/config.toml" event-generator --config /config.toml

docker-run-debug CONFIG="config/example.toml":
    docker run --rm -v "$PWD/{{CONFIG}}:/config.toml" event-generator-debug:latest --config /config.toml
