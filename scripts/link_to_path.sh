#!/bin/bash

# Link all i-rs CLI tools to PATH
# Usage: ./link_to_path.sh

RELEASE_DIR="$(cd "$(dirname "$0")/.." && pwd)/target/release"
BIN_DIR="$HOME/.local/bin"

mkdir -p "$BIN_DIR"

echo "Linking CLI tools from $RELEASE_DIR to $BIN_DIR..."

for binary in "$RELEASE_DIR"/i-rs-*; do
    if [ -f "$binary" ] && [ -x "$binary" ]; then
        name=$(basename "$binary")
        link="$BIN_DIR/$name"

        if [ -L "$link" ]; then
            rm "$link"
            echo "  Updated: $name"
        else
            echo "  Linked: $name"
        fi

        ln -s "$binary" "$link"
    fi
done

echo ""
echo "Done! Add $BIN_DIR to your PATH if not already included:"
echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
echo ""
echo "Or run:"
echo "  source ~/.zshrc  # or source ~/.bashrc"