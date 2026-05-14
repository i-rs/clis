# i-rs-core

Shared core library for all i-rs CLI tools.

## Features

- **Generic Storage** — `Storage<T>` for JSON file persistence with automatic directory creation
- **Output Formatting** — Unified presentation layer with table formatting (tabled), colored output (owo-colors), and JSON output
- **Theme System** — Customizable color themes via `theme.json`
- **Date Utilities** — Flexible date parsing supporting multiple formats
- **Input Validation** — Shared validation functions for names, URLs, weights, and amounts

## Usage

All i-rs CLI crates depend on i-rs-core. Add to your `Cargo.toml`:

```toml
[dependencies]
i-rs-core = { path = "../i-rs-core" }
```

### Storage

```rust
use i_rs_core::Storage;

let mut storage = Storage::<MyData>::new("my-tool");
storage.load()?;
// Work with storage.data ...
storage.save()?;
```

### Output

```rust
use i_rs_core::{
    print_success, print_error, print_header, print_warning,
    output_list, output_item, output_error, OutputFormat,
};
```

### Validation

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, validate_amount};
```

## Modules

| Module | Description |
|--------|-------------|
| `storage` | Generic `Storage<T>` for JSON persistence with `HasTags` trait |
| `presentation` | Output formatting — tables, colors, JSON responses |
| `presentation::output` | JSON response helpers (`ListResponse`, `ItemResponse`, `ErrorResponse`) |
| `presentation::theme` | Customizable color theme system |
| `utils::date` | Date/datetime parsing (`parse_date`, `parse_datetime`) |
| `utils::validation` | Input validation (`validate_name`, `validate_url`, `validate_weight`, `validate_amount`) |

## License

MIT OR Apache-2.0
