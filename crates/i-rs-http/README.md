# i-rs-http

HTTP client CLI tool.

## Install

```bash
npm install -g @i-rs/i-rs-http
# or
brew install i-rs/homebrew-tap/i-rs-http
```

## Usage

```bash
i-rs-http [OPTIONS] <URL>
```

### Options

| Option | Description |
|--------|-------------|
| `-m, --method <METHOD>` | HTTP method: get, post, put, delete (default: get) |
| `-H, --header <KEY:VALUE>` | Custom HTTP header |

## Examples

```bash
# GET request
i-rs-http https://api.example.com

# POST request
i-rs-http -m post https://api.example.com/data

# With headers
i-rs-http -H "Content-Type: application/json" https://api.example.com
```

## License

MIT OR Apache-2.0
