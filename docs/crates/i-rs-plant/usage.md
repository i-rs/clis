# Usage

## Global Flags

- `--json` — Output in JSON format


## Commands

### add

Add a new plant:

```bash
i-rs-plant add --name "Monstera" --species "Monstera deliciosa" --location "Living room" --interval 7
```

Options:
- `--name, -n`: Plant name (required)
- `--species, -s`: Plant species (required)
- `--location, -l`: Plant location (required)
- `--interval, -i`: Watering interval in days (default: 7)
- `--tag, -t`: Add tags (can be repeated)
- `--remark, -r`: Add remarks (can be repeated)

### list

List all plants:

```bash
i-rs-plant list
```

Filter by tag:

```bash
i-rs-plant list --tag indoor
```

### get

Get plant details:

```bash
i-rs-plant get Monstera
```

### water

Record watering:

```bash
i-rs-plant water Monstera
```

### update

Update plant info:

```bash
i-rs-plant update Monstera --location "Bedroom" --interval 10
```

### delete

Delete a plant:

```bash
i-rs-plant delete Monstera
```

### stats

View plant statistics:

```bash
i-rs-plant stats
```

### example

Show usage examples:

```bash
i-rs-plant example
```

### skill

Show AI skill documentation:

```bash
i-rs-plant skill          # Show full content
i-rs-plant skill summary   # Show summary
```

## Global Options

- `--json`: Output in JSON format

### data

Manage data (export, import, clear).

```bash
i-rs-plant data export
i-rs-plant data import [FILE]
i-rs-plant data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
