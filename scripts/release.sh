#!/usr/bin/env bash
set -euo pipefail

if ! command -v git >/dev/null 2>&1; then
  echo "git is required"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required"
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
CURRENT_VERSION="$(grep '^version\s*=\s*"' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"

if [ -z "$CURRENT_VERSION" ]; then
  echo "Unable to parse version from Cargo.toml"
  exit 1
fi

IFS='.' read -r MAJOR MINOR _PATCH <<< "$CURRENT_VERSION"
NEXT_VERSION="${MAJOR}.$((MINOR + 1)).0"
NEXT_TAG="v${NEXT_VERSION}"

if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree has uncommitted changes — commit or stash them before releasing."
  exit 1
fi

echo ""
echo "Current branch:  $CURRENT_BRANCH"
echo "Current version: $CURRENT_VERSION"
echo "Next release:    $NEXT_VERSION  (tag: $NEXT_TAG)"

if git rev-parse "$NEXT_TAG" >/dev/null 2>&1; then
  echo "Tag '$NEXT_TAG' already exists locally."
  exit 1
fi

if git ls-remote --tags origin "refs/tags/${NEXT_TAG}" | grep -q .; then
  echo "Tag '$NEXT_TAG' already exists on origin."
  exit 1
fi

echo ""
read -r -p "Run tests before release? [Y/n]: " RUN_TESTS
RUN_TESTS=${RUN_TESTS:-Y}
if [[ "$RUN_TESTS" =~ ^[Yy]$ ]]; then
  echo "Running cargo test..."
  cargo test
fi

echo ""
HEAD_SHA="$(git rev-parse --short HEAD)"
echo "Release summary"
echo "- branch:  $CURRENT_BRANCH"
echo "- head:    $HEAD_SHA"
echo "- tag:     $NEXT_TAG"
echo ""
read -r -p "Bump to ${NEXT_VERSION}, tag and push? [Y/n]: " CONFIRM
CONFIRM=${CONFIRM:-Y}
if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
  echo "Cancelled."
  exit 0
fi

# Bump Cargo.toml and Cargo.lock
sed -i "0,/^version\s*=\s*\"${CURRENT_VERSION}\"/s//version = \"${NEXT_VERSION}\"/" Cargo.toml
cargo check --quiet 2>/dev/null || true
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${NEXT_VERSION}"
echo "Version bumped to ${NEXT_VERSION}"

# Tag
git tag -a "$NEXT_TAG" -m "Release $NEXT_TAG"
echo "Created local tag: $NEXT_TAG"

# Push
echo ""
read -r -p "Push branch '$CURRENT_BRANCH' and tag '$NEXT_TAG' to origin? [Y/n]: " DO_PUSH
DO_PUSH=${DO_PUSH:-Y}
if [[ "$DO_PUSH" =~ ^[Yy]$ ]]; then
  git push origin "$CURRENT_BRANCH"
  git push origin "$NEXT_TAG"
  echo ""
  echo "Done. GitHub Actions release workflow should start for tag '$NEXT_TAG'."
else
  echo ""
  echo "Tag created locally but not pushed. When ready:"
  echo "  git push origin $CURRENT_BRANCH"
  echo "  git push origin $NEXT_TAG"
fi
