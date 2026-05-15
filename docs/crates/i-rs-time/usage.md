# i-rs-time Usage

## Global Flags

- `--json` — Output in JSON format

## Commands

### start

Start tracking time for a task.

```bash
i-rs-time start <NAME> [OPTIONS]
```

### stop

Stop tracking current task.

```bash
i-rs-time stop
```

### report

Generate time report.

```bash
i-rs-time report [OPTIONS]
```

### list

List time entries.

```bash
i-rs-time list [OPTIONS]
```

### get

Get time entry details.

```bash
i-rs-time get <ID>
```

### delete

Delete a time entry.

```bash
i-rs-time delete <ID>
```

### data

Manage data (export, import, clear).

```bash
i-rs-time data export
i-rs-time data import [FILE]
i-rs-time data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data

### example

Show usage examples.

```bash
i-rs-time example
```

### skill

Show skill information.

```bash
i-rs-time skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/time.json`
- Linux: `~/.config/i-rs/time.json`
- Windows: `~\AppData\Roaming\i-rs\time.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-time list
```
