#!/bin/bash
set -e

echo "Building all targets..."
cargo dist build

echo "Publishing npm packages..."
./scripts/publish-npm.sh

echo "Done!"
