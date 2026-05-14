---
name: "i-rs-core"
description: "Shared core library for i-rs CLI tools. Invoke when working with Storage, output formatting, date parsing, or input validation."
---

# i-rs-core

Shared core library for all i-rs CLI tools.

## Modules

### storage
- `Storage<T>` - Generic JSON file persistence
  - `new(filename)` / `load()` / `save()` / `save_data()`
  - Config dir: `~/.config/i-rs/` (override with `CONFIG_DIR`)
- `HasTags` trait + `filter_by_tag()` helper

### presentation
- Colored output: `print_error`, `print_success`, `print_header`, `print_warning`, `println_dimmed`
- Theme system: `Theme`, `get_theme()`, `apply()`
- Table colors: `table_border_color()`, `table_header_style()`, `table_row_style()`
- JSON output: `OutputFormat`, `output_list()`, `output_item()`, `output_error()`

### utils::date
- `parse_date(&str) -> NaiveDate` - Supports YYYY-MM-DD, YYYY/MM/DD, DD-MM-YYYY, DD/MM/YYYY
- `parse_datetime(&str) -> DateTime<Utc>` - Same formats + time

### utils::validation
- `validate_name(name)` - Non-empty, ≤100 chars, no `/ \ : * ? " < > |`
- `validate_url(url)` - Starts with http:// or https://, ≤2000 chars
- `validate_weight(weight)` - 0 < weight ≤ 1000
- `validate_amount(amount)` - 0 < amount ≤ 1 billion

## Commands

Not a CLI tool — i-rs-core is a library, invoked via `use i_rs_core::*` from other crates.

## Dependencies

```toml
i-rs-core = { path = "../i-rs-core" }
```

## Examples

```rust
use i_rs_core::{Storage, validate_name, print_success};

let mut storage = Storage::<MyData>::new("my-tool");
storage.load()?;
validate_name(&name)?;
print_success("Done!");
```
