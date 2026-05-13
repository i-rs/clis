# i-rs-http Usage Guide

## Install

```bash
npm install -g @i-rs/http
# or
brew install i-rs/http-tap/http
```

## Usage

```bash
i-rs-http [OPTIONS] <URL>
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `-m, --method <METHOD>` | HTTP method | get, post, put, delete (default: get) |
| `-H, --header <KEY:VALUE>` | Custom header | Can be specified multiple times |

## Examples

```bash
# GET request
i-rs-http https://api.example.com

# POST request
i-rs-http -m post https://api.example.com/data

# With headers
i-rs-http -H "Content-Type: application/json" -H "Authorization: Bearer token" https://api.example.com

# PUT request
i-rs-http -m put https://api.example.com/update/1

# DELETE request
i-rs-http -m delete https://api.example.com/delete/1
```

## Output

```
Status: 200 OK
Headers: {...}
Body:
{"message": "success"}
```
