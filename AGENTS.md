# AGENTS.md - i-rs CLI Tools

## Project Overview

Rust monorepo with 7 cross-platform CLI tools: server, password, bookmark, note, domain, remind, weight management.

## Build & Development

```bash
# Build all crates
cargo build

# Run specific tool
cargo run -p i-rs-server -- --help

# Test specific crate
cargo test -p i-rs-server
```

**Note**: Uses Rust edition 2024 (requires nightly or very recent stable).

## Crate Structure (per SPEC.md)

Each crate follows this pattern:
```
crates/i-rs-{name}/
├── src/
│   ├── main.rs        # CLI entry point (clap)
│   ├── commands/      # add, delete, get, list, update
│   ├── models/        # Data structs with serde
│   ├── storage/       # keyring + JSON file
│   └── presentation/  # tabled output
├── Cargo.toml
└── README.md
```

## Key Conventions

- **Workspace deps**: All dependencies defined in root `Cargo.toml`, crates use `.workspace = true`
- **Passwords**: Always store in OS keychain (keyring crate), NEVER in JSON config
- **Data location**: `~/.config/i-rs/` (override with `CONFIG_DIR` env var)
- **Date handling**: chrono with `ts_seconds` serde format
- **Error handling**: `anyhow::Result<()>` with `?` operator
- **CLI framework**: clap with derive macro, snake_case params auto-convert to kebab-case
- **Output**: tabled with cyan headers, green rows (see SPEC.md for exact config)

## Release Process

```bash
# Update version in root Cargo.toml (workspace.package.version)
git tag v0.0.x
git push origin v0.0.x
```

CI (cargo-dist) auto-builds and publishes to:
- GitHub Releases
- npm (`@i-rs/i-rs-*`)
- Homebrew (`i-rs/homebrew-tap/i-rs-*`)

## Important Files

- `SPEC.md` - Detailed project specification (Chinese)
- `Cargo.toml` - Workspace config, edition 2024
- `.github/workflows/release.yml` - CI/release automation

## Common Patterns

- Model structs: `#[derive(Debug, Clone, Serialize, Deserialize)]` + chrono timestamps
- Sensitive fields: `#[serde(skip)]` + `#[allow(dead_code)]`
- Table rows: `#[derive(Tabled)]` with `#[tabled(rename = "NAME")]`