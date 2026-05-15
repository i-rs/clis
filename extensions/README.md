# i-rs Browser Extensions

This directory contains browser extensions that communicate with i-rs CLI tools via Native Messaging.

## Directory Structure

```
extensions/
├── i-rs-kv-chrome/          # Chrome extension for i-rs-kv
│   ├── manifest.json        # Extension manifest (Manifest V3)
│   ├── popup.html/css/js    # Popup UI
│   ├── background.js        # Service worker
│   ├── icons/               # Extension icons (SVG)
│   └── README.md            # Installation instructions
├── i-rs-native-msg/         # Native Messaging host (Rust)
│   ├── src/main.rs          # Host implementation
│   ├── Cargo.toml           # Dependencies
│   └── README.md            # Build & installation
├── install-macos.sh         # One-click installer for macOS
└── install-linux.sh         # One-click installer for Linux
```

## Quick Start

### 1. Build Native Messaging Host

```bash
cd extensions/i-rs-native-msg
cargo build --release
```

### 2. Install the Extension

**macOS:**
```bash
chmod +x extensions/install-macos.sh
./extensions/install-macos.sh
```

**Linux:**
```bash
chmod +x extensions/install-linux.sh
./extensions/install-linux.sh
```

### 3. Load Extension in Browser

**Chrome:**
1. Go to `chrome://extensions/`
2. Enable "Developer mode"
3. Click "Load unpacked"
4. Select `extensions/i-rs-kv-chrome/`
5. Copy the extension ID

**Firefox:**
1. Go to `about:debugging#/runtime/this-firefox`
2. Click "Load Temporary Add-on..."
3. Select `extensions/i-rs-kv-chrome/manifest.json`

### 4. Configure Native Messaging

Update the extension ID in the host manifest:

**macOS Chrome:**
```bash
nano ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.irs.kv.json
```

**Linux Chrome:**
```bash
nano ~/.config/google-chrome/NativeMessagingHosts/com.irs.kv.json
```

Replace `YOUR_EXTENSION_ID_HERE` with your actual extension ID.

### 5. Restart Browser

Close and reopen Chrome/Firefox.

## Architecture

```
┌─────────────────┐     Native Messaging      ┌──────────────────┐
│  Chrome/Firefox │ ◄──── JSON over stdio ────► │  i-rs-native-msg │
│  Extension JS   │                            │  (Rust binary)   │
└─────────────────┘                            └────────┬─────────┘
                                                        │
                                                        │ reads/writes
                                                        ▼
                                               ┌──────────────────┐
                                               │  ~/.config/i-rs/ │
                                               │      kv.json     │
                                               └──────────────────┘
```

## Message Protocol

Messages are JSON objects with an `action` field:

```json
// List all entries
{ "action": "List" }

// Get specific entry
{ "action": "Get", "key": "my-key" }

// Set entry
{ "action": "Set", "key": "my-key", "value": "my-value" }

// Delete entry
{ "action": "Delete", "key": "my-key" }

// Search entries
{ "action": "Search", "query": "search-term" }
```

Response format:
```json
{
  "success": true,
  "message": "OK",
  "data": [...]
}
```

## Security Notes

- Native messaging host runs with user permissions
- `allowed_origins` restricts which extensions can communicate
- No authentication currently (local use only)
- Consider adding input validation for production use
