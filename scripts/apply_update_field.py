#!/usr/bin/env python3
"""Replace simple 'if let Some' field updates with update_field! macro."""

import os
import re

CRATES_DIR = "crates"

pattern = re.compile(
    r'(\s+)if let Some\((\w+)\)\s*=\s*(\w+)\)?\s*\{\s*\n'
    r'\s+(\w+)\.(\w+)\s*=\s*\2\s*;\s*\n'
    r'\s+\}'
)

for entry in sorted(os.listdir(CRATES_DIR)):
    if not entry.startswith("i-rs-"):
        continue
    path = os.path.join(CRATES_DIR, entry, "src", "commands", "update.rs")
    if not os.path.exists(path):
        continue

    with open(path) as f:
        content = f.read()

    c = 0
    while True:
        m = pattern.search(content)
        if not m:
            break
        indent = m.group(1)
        field_name = m.group(3)
        entity = m.group(4)
        target = m.group(5)
        replacement = f"{indent}i_rs_core::update_field!({entity}.{target}, {field_name});"
        content = content[:m.start()] + replacement + content[m.end():]
        c += 1

    if c > 0:
        with open(path, 'w') as f:
            f.write(content)
        print(f"OK   {entry}: {c} replacements")
    else:
        print(f"SKIP {entry}: no simple patterns found")
