# i-rs CLI Tools

A collection of cross-platform CLI tools built with Rust.

## Tools

| Tool | Description |
| -------- | ------------- |
| [i-rs-server](crates/i-rs-server) | Server management CLI |
| [i-rs-password](crates/i-rs-password) | Password management CLI |
| [i-rs-bookmark](crates/i-rs-bookmark) | Bookmark management CLI |
| [i-rs-note](crates/i-rs-note) | Note management CLI |

## Quick Install

```bash
# npm
npm install -g @i-rs/i-rs-server @i-rs/i-rs-password @i-rs/i-rs-bookmark @i-rs/i-rs-note

# Homebrew
brew install i-rs/homebrew-tap/i-rs-server
brew install i-rs/homebrew-tap/i-rs-password
brew install i-rs/homebrew-tap/i-rs-bookmark
brew install i-rs/homebrew-tap/i-rs-note
```

## Development

```bash
# Build all
cargo build

# Run specific tool
cargo run -p i-rs-server -- --help
cargo run -p i-rs-password -- --help
cargo run -p i-rs-bookmark -- --help
cargo run -p i-rs-note -- --help
```

## Release

```bash
# Update version in Cargo.toml, then:
git tag v0.0.x
git push origin v0.0.x
```

CI will automatically build, create GitHub Release, publish to npm and Homebrew.

## Documentation

- [crates/i-rs-server/README.md](crates/i-rs-server/README.md)
- [crates/i-rs-password/README.md](crates/i-rs-password/README.md)
- [crates/i-rs-bookmark/README.md](crates/i-rs-bookmark/README.md)
- [crates/i-rs-note/README.md](crates/i-rs-note/README.md)

## License

MIT OR Apache-2.0
