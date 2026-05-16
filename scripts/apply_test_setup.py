#!/usr/bin/env python3
"""Replace setup() function in tests.rs with test_setup! macro call."""

import os
import re

CRATES_DIR = "crates"

for entry in sorted(os.listdir(CRATES_DIR)):
    if not entry.startswith("i-rs-"):
        continue

    path = os.path.join(CRATES_DIR, entry, "src", "tests.rs")
    if not os.path.exists(path):
        continue

    with open(path) as f:
        lines = f.readlines()

    # Find the setup function start
    setup_start = None
    for i, line in enumerate(lines):
        if line.strip().startswith("fn setup()"):
            setup_start = i
            break

    if setup_start is None:
        print(f"SKIP {entry}: no setup() found")
        continue

    # Verify it matches our pattern by checking next lines
    expected_patterns = [
        'use std::sync::OnceLock',
        'static INIT: OnceLock<()> = OnceLock::new()',
        'INIT.get_or_init(||',
        'let tmp = std::env::temp_dir()',
        '.join(format!',
        '// ensure dir exists',
        'let _ = std::fs::create_dir_all(&tmp)',
        'unsafe { std::env::set_var',
        '});',
    ]

    for j, pattern in enumerate(expected_patterns):
        idx = setup_start + 1 + j
        if idx >= len(lines) or pattern not in lines[idx]:
            print(f"SKIP {entry}: pattern mismatch at line {idx}: expected '{pattern}', got '{lines[idx].strip()}'")
            break
    else:
        # Verify closing } at setup_start + 1 + len(expected_patterns)
        close_idx = setup_start + 1 + len(expected_patterns)
        if close_idx < len(lines) and lines[close_idx].strip() == '}':
            # Remove the setup function and replace with macro call
            new_lines = lines[:setup_start] + [f"    i_rs_core::test_setup!(\"{entry}\");\n"] + lines[close_idx + 1:]
            with open(path, 'w') as f:
                f.writelines(new_lines)
            print(f"OK   {entry}")
        else:
            print(f"SKIP {entry}: no closing brace at line {close_idx}")
