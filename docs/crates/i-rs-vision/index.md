# i-rs-vision

Vision tracking CLI tool for recording and tracking eye prescription measurements over time.

## Features

- Record vision measurements (sphere, cylinder, axis)
- Track left and right eye separately
- View prescription history
- Monitor vision changes over time
- Calculate vision statistics
- Tag and remark support
- JSON output support

## Quick Start

```bash
# Add a vision record
i-rs-vision add 2025-06-14 -ls -3.50 -rs -4.00 -lc -0.50 -rc -0.75 -la 180 -ra 5

# List all records
i-rs-vision list

# View statistics
i-rs-vision stats
```

## Vision Parameters

| Parameter | Short | Description |
|-----------|-------|-------------|
| --left-sphere | -l | Left eye sphere (diopters) |
| --right-sphere | -r | Right eye sphere (diopters) |
| --left-cylinder | -L | Left eye cylinder (diopters) |
| --right-cylinder | -R | Right eye cylinder (diopters) |
| --left-axis | -a | Left eye axis (degrees) |
| --right-axis | -b | Right eye axis (degrees) |

## Data Storage

- macOS: `~/.config/i-rs/visions.json`
- Linux: `~/.config/i-rs/visions.json`
- Windows: `~\AppData\Roaming\i-rs\visions.json`
