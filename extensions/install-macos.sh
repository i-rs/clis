#!/bin/bash
set -e

echo "=== i-rs Chrome Extension Installer ==="

# Build the native messaging host
echo "Building i-rs-native-msg..."
cd "$(dirname "$0")/i-rs-native-msg"
cargo build --release

# Get the binary path
BINARY_PATH="$(pwd)/target/release/i-rs-native-msg"

# Install binary
echo "Installing to /usr/local/bin..."
sudo cp "$BINARY_PATH" /usr/local/bin/

# Create Chrome native messaging host manifest
CHROME_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
mkdir -p "$CHROME_DIR"

# Ask for extension ID
read -p "Enter your Chrome extension ID (or press Enter to skip): " EXTENSION_ID

if [ -n "$EXTENSION_ID" ]; then
    cat > "$CHROME_DIR/com.irs.kv.json" << EOF
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF
    echo "Created manifest at $CHROME_DIR/com.irs.kv.json"
else
    cat > "$CHROME_DIR/com.irs.kv.json" << 'EOF'
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID_HERE/"
  ]
}
EOF
    echo "Created template manifest - UPDATE the extension ID!"
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "1. Load the extension in Chrome: chrome://extensions/"
echo "2. Copy your extension ID from the extension card"
echo "3. Update allowed_origins in $CHROME_DIR/com.irs.kv.json"
echo "4. Restart Chrome"
echo ""
echo "Data is stored at: ~/.config/i-rs/kv.json"
