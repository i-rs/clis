# i-rs-appliance Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new appliance to track.

```bash
i-rs-appliance add <NAME> <BRAND> <MODEL> <PURCHASE_DATE> <LIFESPAN_YEARS> [OPTIONS]
```

**Arguments:**
- `NAME` - Appliance name
- `BRAND` - Brand name
- `MODEL` - Model number
- `PURCHASE_DATE` - Purchase date (YYYY-MM-DD)
- `LIFESPAN_YEARS` - Expected lifespan in years

**Options:**
- `-t, --tag <TAG>` - Add tags (repeatable)
- `-r, --remark <REMARK>` - Add remarks (repeatable)

**Example:**
```bash
i-rs-appliance add "Refrigerator" Samsung "RF28R7551" 2020-01-15 10 --tag kitchen --tag electronics
```

### list

List all appliances with status.

```bash
i-rs-appliance list [OPTIONS]
```

**Options:**
- `-t, --tag <TAG>` - Filter by tag

**Example:**
```bash
i-rs-appliance list
i-rs-appliance list --tag kitchen
```

### get

Show detailed appliance information.

```bash
i-rs-appliance get <NAME>
```

**Example:**
```bash
i-rs-appliance get "Refrigerator"
```

### update

Update appliance or add maintenance record.

```bash
i-rs-appliance update <NAME> [OPTIONS]
```

**Options:**
- `-b, --brand <BRAND>` - Update brand
- `-m, --model <MODEL>` - Update model
- `-l, --lifespan <YEARS>` - Update lifespan years
- `-t, --tag <TAG>` - Update tags
- `-r, --remark <REMARK>` - Update remarks
- `--add-maintenance <DESC>` - Add maintenance record description
- `--maintenance-date <DATE>` - Maintenance record date (YYYY-MM-DD)

**Example:**
```bash
i-rs-appliance update "Refrigerator" --lifespan 12
i-rs-appliance update "Refrigerator" --add-maintenance "Replaced water filter"
```

### delete

Remove an appliance.

```bash
i-rs-appliance delete <NAME>
```

**Example:**
```bash
i-rs-appliance delete "Old Microwave"
```

### stats

Show statistics overview.

```bash
i-rs-appliance stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-appliance data export
i-rs-appliance data import [FILE]
i-rs-appliance data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-appliance example
```

### skill

Show skill information.

```bash
i-rs-appliance skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/appliances.json`
- Linux: `~/.config/i-rs/appliances.json`
- Windows: `~\AppData\Roaming\i-rs\appliances.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-appliance list
```
