#!/bin/bash

# Generate shell completions for all i-rs CLI tools
# Usage: ./generate_completions.sh [bash|zsh|fish|all]

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$SCRIPT_DIR/target/release"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "Building project first..."
    cargo build --release
fi

BASH_DIR="$HOME/.local/share/bash-completion/completions"
ZSH_DIR="$HOME/.local/share/zsh/site-functions"
FISH_DIR="$HOME/.config/fish/completions"

install_bash() {
    echo "Installing bash completions to $BASH_DIR..."
    mkdir -p "$BASH_DIR"
    for binary in "$RELEASE_DIR"/i-rs-*; do
        if [ -f "$binary" ] && [ -x "$binary" ]; then
            name=$(basename "$binary")
            "$binary" generate-completions bash > "$BASH_DIR/$name" 2>/dev/null || true
            echo "  $name"
        fi
    done
}

install_zsh() {
    echo "Installing zsh completions to $ZSH_DIR..."
    mkdir -p "$ZSH_DIR"
    for binary in "$RELEASE_DIR"/i-rs-*; do
        if [ -f "$binary" ] && [ -x "$binary" ]; then
            name=$(basename "$binary")
            "$binary" generate-completions zsh > "$ZSH_DIR/_$name" 2>/dev/null || true
            echo "  $name"
        fi
    done
}

install_fish() {
    echo "Installing fish completions to $FISH_DIR..."
    mkdir -p "$FISH_DIR"
    for binary in "$RELEASE_DIR"/i-rs-*; do
        if [ -f "$binary" ] && [ -x "$binary" ]; then
            name=$(basename "$binary")
            "$binary" generate-completions fish > "$FISH_DIR/$name.fish" 2>/dev/null || true
            echo "  $name"
        fi
    done
}

case "${1:-all}" in
    bash)
        install_bash
        echo ""
        echo "Add to ~/.bashrc:"
        echo '  export XDG_DATA_DIRS="$HOME/.local/share:$XDG_DATA_DIRS"'
        ;;
    zsh)
        install_zsh
        echo ""
        echo "Zsh completions installed! If not working, add to ~/.zshrc:"
        echo '  export fpath=("$HOME/.local/share/zsh/site-functions" $fpath)'
        echo "  autoload -Uz compinit && compinit"
        ;;
    fish)
        install_fish
        ;;
    all)
        install_bash
        echo ""
        install_zsh
        echo ""
        install_fish
        ;;
    *)
        echo "Usage: $0 [bash|zsh|fish|all]"
        exit 1
        ;;
esac

echo ""
echo "Done! Restart your shell or run:"
echo "  source ~/.zshrc  # or source ~/.bashrc"