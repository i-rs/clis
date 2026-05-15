# i-rs

Unified CLI dispatcher for all i-rs tools. Run any i-rs tool as a subcommand.

## Overview

`i-rs` is a lightweight dispatcher that follows the same pattern as `git`, `cargo`, `docker`, and `kubectl` for plugin systems. When you run `i-rs todo list`, it looks for `i-rs-todo` in your PATH and forwards all arguments.

Each tool can also be used independently (e.g., `i-rs-todo list`), but the unified interface provides a single entry point for all tracking functionality.

## Features

- Single entry point for 70+ CLI tools
- Automatic tool discovery via PATH lookup
- Same interface as direct tool invocation
- Version information via `--version`
- Tool listing via `--help` or no arguments

## Categories

- **Core & System**: server, password, keys, kv, deploy
- **Notes & Bookmarks**: note, bookmark, article, read, quote, spark, snippet, vocab
- **Health & Fitness**: weight, height, mood, sleep, step, water, cal, fast, exercise, run, cycling, dose, allergy, sit, vision
- **Finance**: ledger, budget, recur, sub, invest, debt, invoice, tax, goal
- **Reminders & Expiry**: remind, domain, bestby, tick, time, event, birthday
- **Home & Care**: sheet, toothbrush, towel, bed, ac, filter, purify, appliance, plant
- **Pets**: feedpet, petbath, walkdog, aqua
- **Productivity**: todo, habit, project, want, gift, car, meal, pig, grocery, contact, movie, podcast, cycle
