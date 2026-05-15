# i-rs-native-msg

Native Messaging host for i-rs Chrome Extension.

This binary provides communication between the Chrome extension and i-rs key-value storage.

## Build

```bash
cargo build --release
```

The binary will be at `target/release/i-rs-native-msg`.

## Installation

### macOS / Linux

1. Build the binary:
```bash
cargo build --release
```

2. Copy the binary to a location in your PATH:
```bash
sudo cp target/release/i-rs-native-msg /usr/local/bin/
```

3. Create the host manifest file:

For **Chrome** on macOS:
```bash
mkdir -p ~/Library/Application Support/Google/Chrome/NativeMessagingHosts/
cat > ~/Library/Application\ Support/Google/Chrome/NativeMessagingHosts/com.irs.kv.json << 'EOF'
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_origins": [
    "chrome-extension://EXTENSION_ID_HERE/"
  ]
}
EOF
```

For **Firefox**:
```bash
mkdir -p ~/.mozilla/native-messaging-hosts/
cat > ~/.mozilla/native-messaging-hosts/com.irs.kv.json << 'EOF'
{
  "name": "com.irs.kv",
  "description": "i-rs KV Native Messaging Host",
  "path": "/usr/local/bin/i-rs-native-msg",
  "allowed_extensions": [
    "EXTENSION_ID_HERE@mozilla.org"
  ]
}
EOF
```

### Windows

1. Build the binary:
```bash
cargo build --release
```

2. Copy to PATH:
```bash
copy target\release\i-rs-native-msg.exe C:\Windows\System32\
```

3. Create registry entry (run as Administrator):
```reg
Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SOFTWARE\Google\Chrome\NativeMessagingHosts\com.irs.kv]
@="C:\\Windows\\System32\\i-rs-native-msg-manifest.json"
```

4. Create the manifest file at `C:\Windows\System32\i-rs-native-msg-manifest.json`.

## Getting Extension ID

After loading the extension in Chrome:
1. Go to `chrome://extensions/`
2. Find your extension
3. The extension ID is shown at the top

## Data Storage

The native host stores data at:
- macOS: `~/.config/i-rs/kv.json`
- Linux: `~/.config/i-rs/kv.json`
- Windows: `%APPDATA%\i-rs\kv.json`

## Supported Actions

| Action | Description |
|--------|-------------|
| `List` | List all key-value pairs |
| `Get` | Get a specific value by key |
| `Set` | Set a value for a key |
| `Delete` | Delete a key |
| `Search` | Search keys and values |
