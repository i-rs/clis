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
- `--tag, -t` - Add tags (repeatable)
- `--remark, -r` - Add remarks (repeatable)

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
- `--tag, -t` - Filter by tag

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
- `--brand, -b` - Update brand
- `--model, -m` - Update model
- `--lifespan, -l` - Update lifespan years
- `--tag, -t` - Update tags
- `--remark, -r` - Update remarks
- `--add-maintenance` - Add maintenance record description
- `--maintenance-date` - Maintenance record date (YYYY-MM-DD)

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

### Global Options

- `--json` - Output in JSON format

### example

Show usage examples.

```bash
i-rs-appliance example
```

### skill

View AI skill documentation.

```bash
i-rs-appliance skill
i-rs-appliance skill summary
i-rs-appliance skill content
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
