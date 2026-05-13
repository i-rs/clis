# i-rs CLI Tools

A collection of cross-platform CLI tools built with Rust.

## Tools

| Tool | Description |
|------|-------------|
| [i-rs-hello](crates/i-rs-hello) | Simple hello world CLI |
| [i-rs-fs](crates/i-rs-fs) | File system operations |
| [i-rs-http](crates/i-rs-http) | HTTP client CLI |
| [i-rs-server](crates/i-rs-server) | Server management CLI |

## Quick Install

```bash
# npm
npm install -g @i-rs/i-rs-hello @i-rs/i-rs-fs @i-rs/i-rs-http @i-rs/i-rs-server

# Homebrew
brew install i-rs/homebrew-tap/i-rs-hello
brew install i-rs/homebrew-tap/i-rs-fs
brew install i-rs/homebrew-tap/i-rs-http
brew install i-rs/homebrew-tap/i-rs-server
```

## Development

```bash
# Build all
cargo build

# Run specific tool
cargo run -p i-rs-hello -- --help
cargo run -p i-rs-fs -- --help
cargo run -p i-rs-http -- --help
cargo run -p i-rs-server -- --help
```

## Release

```bash
# Update version in Cargo.toml, then:
git tag v0.0.x
git push origin v0.0.x
```

CI will automatically build, create GitHub Release, publish to npm and Homebrew.

## Documentation

- [crates/i-rs-hello/README.md](crates/i-rs-hello/README.md)
- [crates/i-rs-fs/README.md](crates/i-rs-fs/README.md)
- [crates/i-rs-http/README.md](crates/i-rs-http/README.md)
- [crates/i-rs-server/README.md](crates/i-rs-server/README.md)

## License

MIT OR Apache-2.0
