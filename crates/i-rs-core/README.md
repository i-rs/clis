# i-rs-core

Shared core library for all i-rs CLI tools.

## Features

- Generic JSON-based storage abstraction (`Storage<T>`)
- Common utilities (date parsing, input validation)
- Unified presentation layer (table formatting, colored output, JSON output)
- Error handling helpers

## Modules

- `storage` - Generic `Storage<T>` for JSON file persistence with keyring support
- `presentation` - Output formatting with `tabled`, `owo-colors`, and JSON support
  - `output` - JSON output helpers (`output_list`, `output_item`, `output_error`)
  - `theme` - Table theme configuration
- `utils` - Shared utilities
  - `date` - Date parsing (`parse_date`)
  - `validation` - Input validation (`validate_name`, `validate_url`, `validate_weight`, `ValidationError`)

## Usage

Add to your `Cargo.toml`:

```toml
i-rs-core = { path = "../i-rs-core" }
```

## Storage Pattern

```rust
use i_rs_core::Storage;

let mut storage = Storage::<MyData>::new("my-tool");
storage.load()?;
// ... use storage.data ...
storage.save_data(&storage.data)?;
```

## License

MIT OR Apache-2.0