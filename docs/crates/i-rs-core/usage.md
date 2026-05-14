# i-rs-core Usage

## Storage

### Basic Storage

```rust
use i_rs_core::Storage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct MyData {
    entries: Vec<String>,
}

let mut storage = Storage::<MyData>::new("my-tool");
storage.load()?;
storage.data.entries.push("hello".to_string());
storage.save()?;
```

### With Tags

```rust
use i_rs_core::{Storage, HasTags, filter_by_tag};

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    name: String,
    tags: Vec<String>,
}

impl HasTags for Item {
    fn tags(&self) -> &[String] {
        &self.tags
    }
}

// Filter items by tag
let items = vec![/* ... */];
let filtered = filter_by_tag(&items, Some("work"));
```

### Config Directory

Data is stored at `~/.config/i-rs/{name}.json`. Override with `CONFIG_DIR`:

```bash
CONFIG_DIR=/tmp/config i-rs-todo list
```

## Output Formatting

### Colored Output

```rust
use i_rs_core::{
    print_error,    // Red "Error: ..." to stderr
    print_success,  // Green success message
    print_header,   // Bold cyan header
    print_warning,  // Yellow warning
};

print_success("Operation completed!");
print_error("Something went wrong");
```

### JSON Output

```rust
use i_rs_core::{
    output_list,    // Vec<T> -> pretty JSON
    output_item,    // Single item -> pretty JSON
    output_error,   // Error -> JSON
    OutputFormat,
};

let json = output_list(&items, items.len(), filter.as_deref(), OutputFormat::Json);
println!("{}", json);
```

**Response Formats:**

List:
```json
{
  "success": true,
  "data": [...],
  "meta": { "count": 10, "filter": "work" }
}
```

Item:
```json
{
  "success": true,
  "data": { ... }
}
```

Error:
```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Entry 'xxx' not found"
  }
}
```

## Date Parsing

```rust
use i_rs_core::{parse_date, parse_datetime};

// Supported formats: YYYY-MM-DD, YYYY/MM/DD, DD-MM-YYYY, DD/MM/YYYY
let date = parse_date("2024-01-15")?;

// With time: YYYY-MM-DD HH:MM:SS, YYYY-MM-DD HH:MM
let datetime = parse_datetime("2024-01-15 14:30")?;
```

## Input Validation

```rust
use i_rs_core::{validate_name, validate_url, validate_weight, validate_amount};

validate_name("my-entry")?;        // ≤100 chars, no invalid chars
validate_url("https://example.com")?;  // http(s)://, ≤2000 chars
validate_weight(70.5)?;            // 0 < weight ≤ 1000
validate_amount(100.0)?;           // 0 < amount ≤ 1 billion
```

## Theme Customization

Create `~/.config/i-rs/theme.json` to customize colors:

```json
{
  "error": "bold red",
  "success": "green",
  "header": "bold cyan",
  "warning": "yellow",
  "table_border": "cyan",
  "table_header": "bold cyan",
  "table_row": "green",
  "dimmed": "bright black"
}
```

Supported colors: `red`, `green`, `blue`, `cyan`, `magenta`/`purple`, `yellow`, `white`, `black`. Prefix with `bright` for bright variants. Modifiers: `bold`, `dim`/`dimmed`.
