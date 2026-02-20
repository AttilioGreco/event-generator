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
DEFAULT_TAG="v${CURRENT_VERSION}"

if [ -z "$CURRENT_VERSION" ]; then
  echo "Unable to parse version from Cargo.toml"
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree has changes."
  echo "You can commit them during this release flow."
else
  echo "Working tree is clean."
fi

echo ""
echo "Current branch: $CURRENT_BRANCH"
echo "Current Cargo version: $CURRENT_VERSION"

read -r -p "Run tests before release? [Y/n]: " RUN_TESTS
RUN_TESTS=${RUN_TESTS:-Y}
if [[ "$RUN_TESTS" =~ ^[Yy]$ ]]; then
  echo "Running cargo test..."
  cargo test
fi

echo ""
read -r -p "Create commit with current changes? [y/N]: " DO_COMMIT
DO_COMMIT=${DO_COMMIT:-N}
if [[ "$DO_COMMIT" =~ ^[Yy]$ ]]; then
  DEFAULT_MSG="chore(release): prepare ${DEFAULT_TAG}"
  read -r -p "Commit message [${DEFAULT_MSG}]: " COMMIT_MSG
  COMMIT_MSG=${COMMIT_MSG:-$DEFAULT_MSG}
  git add -A
  if ! git diff --cached --quiet; then
    git commit -m "$COMMIT_MSG"
  else
    echo "Nothing staged; skipping commit."
  fi
fi

echo ""
read -r -p "Tag to create [${DEFAULT_TAG}]: " TAG
TAG=${TAG:-$DEFAULT_TAG}

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Tag '$TAG' already exists locally."
  exit 1
fi

if git ls-remote --tags origin "refs/tags/${TAG}" | grep -q .; then
  echo "Tag '$TAG' already exists on origin."
  exit 1
fi

HEAD_SHA="$(git rev-parse --short HEAD)"
echo ""
echo "Release summary"
echo "- branch: $CURRENT_BRANCH"
echo "- head:   $HEAD_SHA"
echo "- tag:    $TAG"

echo ""
read -r -p "Create annotated tag now? [Y/n]: " CONFIRM_TAG
CONFIRM_TAG=${CONFIRM_TAG:-Y}
if [[ ! "$CONFIRM_TAG" =~ ^[Yy]$ ]]; then
  echo "Cancelled."
  exit 0
fi

git tag -a "$TAG" -m "Release $TAG"
echo "Created local tag: $TAG"

echo ""
read -r -p "Push branch '$CURRENT_BRANCH' and tag '$TAG' to origin? [Y/n]: " DO_PUSH
DO_PUSH=${DO_PUSH:-Y}
if [[ "$DO_PUSH" =~ ^[Yy]$ ]]; then
  git push origin "$CURRENT_BRANCH"
  git push origin "$TAG"
  echo ""
  echo "Done. GitHub Action release workflow should start for tag '$TAG'."
else
  echo ""
  echo "Tag created locally but not pushed."
  echo "When ready:"
  echo "  git push origin $CURRENT_BRANCH"
  echo "  git push origin $TAG"
fi
