# i-rs KV Chrome Extension

A modern Chrome extension for managing key-value storage via the i-rs API.

## Features

- 🔗 Connect to i-rs API (`http://localhost:8080`)
- ➕ Create/Edit/Delete KV entries
- 🔍 Search entries
- 📋 Copy values to clipboard
- 🌙 Dark theme UI
- 💾 Persistent settings (API URL stored in browser)

## Setup

1. **Start i-rs API server:**
   ```bash
   cd /Users/mankong/volumes/code/i-rs/clis
   cargo run -p i-rs-api
   ```

2. **Install dependencies:**
   ```bash
   pnpm install
   ```

3. **Build:**
   ```bash
   pnpm build
   ```

4. **Load in Chrome:**
   - Open `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `dist` folder

## Development

```bash
pnpm dev
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/kv` | List all entries |
| GET | `/api/kv?search=xxx` | Search entries |
| GET | `/api/kv/:key` | Get specific entry |
| POST | `/api/kv/:key` | Create/Update entry |
| DELETE | `/api/kv/:key` | Delete entry |
