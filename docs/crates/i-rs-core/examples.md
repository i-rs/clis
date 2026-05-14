# i-rs-core Examples

## Storage Example

```rust
use i_rs_core::Storage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct TodoList {
    items: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let mut store = Storage::<TodoList>::new("todos");
    store.load()?;

    store.data.items.push("Buy milk".to_string());
    store.data.items.push("Walk dog".to_string());

    store.save()?;

    println!("Saved {} todos", store.data.items.len());
    Ok(())
}
```

## Validation Example

```rust
use i_rs_core::validate_name;

fn create_entry(name: &str) -> anyhow::Result<()> {
    validate_name(name).map_err(|e| anyhow::anyhow!("{}", e.message))?;
    println!("Entry '{}' is valid", name);
    Ok(())
}
```

## Custom Table Output

While each crate has its own `presentation/mod.rs`, the shared theme utilities enable consistent table formatting:

```rust
use i_rs_core::{
    table_border_color, table_header_style, table_row_style,
};
use tabled::{
    Table, settings::{
        Style, style::BorderColor, object::{Rows, Segment},
        themes::Colorization,
    },
};

let table = Table::new(&rows)
    .with(Style::modern_rounded())
    .modify(Segment::all(), BorderColor::filled(table_border_color()))
    .with(Colorization::exact([table_header_style()], Rows::first()))
    .with(Colorization::exact([table_row_style()], Rows::new(1..)))
    .to_string();
```

## Date Parsing Examples

```rust
use i_rs_core::{parse_date, parse_datetime};

// All these work:
parse_date("2024-01-15")?;
parse_date("2024/01/15")?;
parse_date("15-01-2024")?;
parse_date("15/01/2024")?;

// Datetime variants:
parse_datetime("2024-01-15 14:30:00")?;
parse_datetime("2024-01-15 14:30")?;
parse_datetime("2024-01-15")?;  // defaults to 00:00:00 UTC
```

## JSON Output in CLI

Typical CLI command flow mixing table and JSON output:

```rust
use i_rs_core::{
    print_header, print_warning,
    output_list, OutputFormat,
};

fn list_items(format: OutputFormat) -> anyhow::Result<()> {
    let items = get_items()?;

    if items.is_empty() {
        if matches!(format, OutputFormat::Json) {
            println!("{}", output_list::<serde_json::Value>(
                &[], 0, None, format
            ));
        } else {
            print_warning("No items found.");
        }
        return Ok(());
    }

    if matches!(format, OutputFormat::Json) {
        println!("{}", output_list(&items, items.len(), None, format));
    } else {
        print_header("Items");
        display_table(&items);
    }

    Ok(())
}
```
