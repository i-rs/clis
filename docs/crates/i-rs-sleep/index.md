# i-rs-sleep

Sleep tracking CLI tool for recording bedtime, wake time, and sleep quality.

## Overview

i-rs-sleep helps you track your sleep patterns. Record when you go to bed and wake up, rate the quality of your sleep, and view statistics.

## Quick Start

```bash
# Record a sleep session
i-rs-sleep add 22:30 06:45 4 --tag workday

# View all sleep records
i-rs-sleep list

# Check your sleep statistics
i-rs-sleep stats
```

## Key Features

- **Sleep Quality Rating**: Rate your sleep on a scale of 1-5
- **Statistics**: View average sleep duration, quality, and trends
- **Tag Support**: Organize sleep records with tags
- **JSON Output**: Use `--json` flag for programmatic access

## Quality Scale

| Rating | Emoji | Description |
|--------|-------|-------------|
| 1 | 😴 | Awful |
| 2 | 😪 | Poor |
| 3 | 😌 | Fair |
| 4 | 😊 | Good |
| 5 | 😁 | Excellent |