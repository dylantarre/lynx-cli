#!/bin/bash
set -e

# Check if version is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.1"
  exit 1
fi

VERSION=$1
CURRENT_DIR=$(pwd)
SCRIPT_DIR=$(dirname "$0")
PROJECT_DIR="$SCRIPT_DIR/.."

cd "$PROJECT_DIR"

# Ensure we're in the project root
if [ ! -f "Cargo.toml" ]; then
  echo "Error: Could not find Cargo.toml. Make sure you're running this script from the project root or scripts directory."
  exit 1
fi

# Update version in Cargo.toml
echo "Updating version to $VERSION in Cargo.toml..."
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Build the project to ensure it compiles
echo "Building project..."
cargo build --release

# Run tests to ensure everything works
echo "Running tests..."
cargo test

# Update git
echo "Committing version changes..."
git add Cargo.toml
git commit -m "Bump version to $VERSION"

# Create and push git tag
echo "Creating and pushing git tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin "v$VERSION"
git push

echo "Release process initiated for v$VERSION"
echo "GitHub Actions workflow will handle the rest of the deployment process."
echo "Remember to set up the following secrets in your GitHub repository:"
echo "  - CRATES_IO_TOKEN: Your crates.io API token"
echo "  - DOCKERHUB_USERNAME: Your Docker Hub username"
echo "  - DOCKERHUB_TOKEN: Your Docker Hub access token" 