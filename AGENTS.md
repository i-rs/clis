# AGENTS.md - i-rs CLI Tools

## Project Overview

Rust monorepo with 9 cross-platform CLI tools for personal data management.

## Tools (9 Total)

| Tool | Description | Special Commands |
|------|-------------|-----------------|
| i-rs-server | Server management | suggest |
| i-rs-password | Password management | - |
| i-rs-bookmark | Bookmark management | - |
| i-rs-note | Note management | - |
| i-rs-domain | Domain expiry tracking | - |
| i-rs-remind | Event reminders | done |
| i-rs-weight | Weight tracking | chart, stats |
| i-rs-mood | Mood tracking | calendar |
| i-rs-todo | Todo tracking | done |

## Build & Development

```bash
# Build all crates
cargo build

# Build specific tool
cargo build -p i-rs-mood

# Run specific tool
cargo run -p i-rs-mood -- --help

# Test specific crate
cargo test -p i-rs-mood
```

## Crate Structure

Each crate follows this pattern:
```
crates/i-rs-{name}/
├── src/
│   ├── main.rs           # CLI entry point (clap)
│   ├── commands/         # add, delete, get, list, update, example, skill, [special]
│   ├── models/           # Data structs with serde + tabled
│   ├── storage/          # keyring + JSON file
│   └── presentation/     # tabled output, colors, charts, output (JSON formatting)
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
- **Output**: tabled with cyan headers, green rows
- **JSON output**: All commands support `--json` flag for JSON output

## New Crate Workflow

When creating a new crate `i-rs-{name}`:

1. **Create directories**
```bash
mkdir -p crates/i-rs-{name}/src/{models,storage,commands,presentation}
mkdir -p docs/crates/i-rs-{name}
mkdir -p skills/i-rs-{name}
```

2. **Create files** (see SPEC.md for details):
   - `Cargo.toml` with workspace dependencies
   - `src/models/mod.rs` - Entity struct + Row struct (Tabled)
   - `src/storage/mod.rs` - JSON persistence + keyring
   - `src/presentation/mod.rs` - Table formatting
   - `src/presentation/output.rs` - JSON output formatting
   - `src/commands/*.rs` - Command handlers (add, delete, get, list, update, example, skill)
   - `src/main.rs` - CLI parsing with --json global flag

3. **Update configs**:
   - Add to `Cargo.toml` workspace members
   - Add sidebar entry to `docs/.vitepress/config.ts`

4. **Create docs** (in `docs/crates/i-rs-{name}/`):
   - `index.md` - Overview, quick start
   - `usage.md` - Command reference
   - `examples.md` - Usage examples
   - `test.md` - Test records

5. **Create skills** (in `skills/i-rs-{name}/`):
   - `SKILL.md` - AI skill documentation

6. **Build and verify**
```bash
cargo build -p i-rs-{name}
```

## Common Patterns

### Model Struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### Table Row
```rust
#[derive(Tabled)]
pub struct EntityRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
}

impl EntityRow {
    pub fn from_entity(entity: &Entity) -> Self {
        Self {
            name: entity.name.clone(),
            created_at: entity.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
```

### Sensitive Fields
```rust
#[serde(skip)]
#[allow(dead_code)]
pub password: Option<String>,
```

### Time Calculation (for domain/remind/weight/mood)
```rust
pub fn days_until(&self) -> i64 {
    (self.event_date - Utc::now()).num_days()
}
```

### JSON Output Module (presentation/output.rs)
```rust
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Debug, Serialize)]
pub struct ListMeta {
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

pub fn output_list<T: Serialize + Clone>(items: &[T], count: usize, filter: Option<&str>, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            let response = ListResponse {
                success: true,
                data: items.to_vec(),
                meta: ListMeta { count, filter: filter.map(String::from) },
            };
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| r#"{"success":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize"}}"#.to_string())
        }
        OutputFormat::Table => {
            serde_json::to_string(items).unwrap_or_default()
        }
    }
}
```

## Global Commands (All Crates)

### example Command
Show usage examples for AI/human to quickly understand the CLI.
```bash
i-rs-{name} example
```

### skill Command
View AI skill documentation integrated into CLI itself.
```bash
i-rs-{name} skill          # Show raw skill document
i-rs-{name} skill summary  # Show summary
i-rs-{name} skill content  # Show content
```

## JSON Output

All commands support `--json` global flag for JSON output:

```bash
i-rs-{name} list --json
i-rs-{name} get <name> --json
```

### JSON Response Format

**List Response:**
```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 10,
    "filter": "work"
  }
}
```

**Item Response:**
```json
{
  "success": true,
  "data": {...}
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

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
- `AGENTS.md` - This file (development workflow for AI)
- `Cargo.toml` - Workspace config
- `docs/.vitepress/config.ts` - Documentation sidebar config
- `.github/workflows/release.yml` - CI/release automation

## VitePress Documentation

Documentation at `docs/` uses VitePress. Each crate has 4 pages in `docs/crates/i-rs-{name}/`:
- `index.md` - Overview
- `usage.md` - Command reference
- `examples.md` - Detailed examples
- `test.md` - Test records