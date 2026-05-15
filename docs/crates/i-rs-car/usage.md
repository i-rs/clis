# i-rs-car Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### add

Add a new vehicle.

```bash
i-rs-car add <NAME> [OPTIONS]
```

**Arguments:**
- `NAME` - Car name

**Options:**
- `-l, --license-plate <PLATE>` - License plate number
- `-b, --brand <BRAND>` - Car brand
- `-m, --model <MODEL>` - Car model
- `-i, --mileage <KM>` - Current mileage

### list

List all vehicles.

```bash
i-rs-car list [OPTIONS]
```

**Options:**
- `--car <NAME>` - Filter by car name

### fuel

Add fuel record.

```bash
i-rs-car fuel <NAME> [OPTIONS]
```

### maintain

Add maintenance record.

```bash
i-rs-car maintain <NAME> [OPTIONS]
```

### get

Get vehicle details.

```bash
i-rs-car get <NAME>
```

### delete

Delete a vehicle.

```bash
i-rs-car delete <NAME>
```

### stats

Show vehicle statistics.

```bash
i-rs-car stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-car data export
i-rs-car data import [FILE]
i-rs-car data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-car example
```

### skill

Show skill information.

```bash
i-rs-car skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/cars.json`
- Linux: `~/.config/i-rs/cars.json`
- Windows: `~\AppData\Roaming\i-rs\cars.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-car list
```
