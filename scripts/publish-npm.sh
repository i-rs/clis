#!/bin/bash
set -e

DIST_DIR="target/distrib"
PACKAGES=("i-rs-hello" "i-rs-fs" "i-rs-http")

for pkg in "${PACKAGES[@]}"; do
  echo "Publishing $pkg..."

  tar -xzf "$DIST_DIR/$pkg-npm-package.tar.gz" -C "$DIST_DIR/$pkg-npm-package" --strip-components=1

  cd "$DIST_DIR/$pkg-npm-package"
  npm publish --access public
  cd - > /dev/null

  echo "$pkg published successfully!"
done

echo "All npm packages published!"
