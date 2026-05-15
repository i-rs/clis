# i-rs KV Chrome Extension

A Chrome extension that provides a popup interface for managing key-value storage using the i-rs-native-msg Native Messaging host.

## Features

- **Quick Access**: Click the extension icon to open the popup
- **Add Key-Value Pairs**: Easily add new entries
- **Search**: Filter entries by key or value
- **Copy to Clipboard**: One-click copy value
- **Delete Entries**: Remove unwanted entries

## Installation

### 1. Install Native Messaging Host

Follow the instructions in `../i-rs-native-msg/README.md` to install the native host.

### 2. Load the Extension in Chrome

1. Open Chrome and go to `chrome://extensions/`
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked"
4. Select the `extensions/i-rs-kv-chrome` directory
5. Note your extension ID (shown at the top of the extension card)

### 3. Update Native Messaging Host Manifest

Update the `allowed_origins` in the host manifest file:

**macOS Chrome**:
```bash
cat > ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.irs.kv.json << 'EOF'
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

**Linux Chrome**:
```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts/
cat > ~/.config/google-chrome/NativeMessagingHosts/com.irs.kv.json << 'EOF'
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

**Windows Chrome**: Edit the registry entry to include your extension ID.

### 4. Restart Chrome

Close and reopen Chrome for the native messaging changes to take effect.

## Usage

1. Click the i-rs KV icon in the Chrome toolbar
2. Add entries using the input fields at the top
3. Search through entries using the search bar
4. Click "Copy" to copy a value to clipboard
5. Click "×" to delete an entry

## File Structure

```
i-rs-kv-chrome/
├── manifest.json      # Extension manifest
├── background.js      # Service worker
├── popup.html        # Popup UI
├── popup.css         # Popup styles
├── popup.js          # Popup logic
├── background.js     # Background script
└── icons/
    ├── icon16.png
    ├── icon48.png
    └── icon128.png
```

## Troubleshooting

### "Native messaging host not found"

1. Verify the host binary is in your PATH: `which i-rs-native-msg`
2. Check the manifest file exists and is valid JSON
3. Make sure the extension ID in `allowed_origins` matches your extension
4. Restart Chrome

### "Native host exited with error"

Run the native host manually to see error output:
```bash
echo '{}' | i-rs-native-msg
```

### Extension not responding

Check Chrome's developer console (View > Developer > JavaScript Console) for errors.
