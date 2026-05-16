#!/usr/bin/env python3
"""Replace empty-result pattern in list.rs with handle_empty! macro call."""

import os
import re

CRATES_DIR = "crates"

def process_crate(entry):
    """Process one crate's list.rs, return True if modified."""
    path = os.path.join(CRATES_DIR, entry, "src", "commands", "list.rs")
    if not os.path.exists(path):
        return False

    with open(path) as f:
        lines = f.readlines()

    # Find empty-result block. Pattern spans multiple lines:
    # Line i:   '    if XXX.is_empty() {'
    # Line i+1: '        if format.is_json() {'
    # Line i+2: '            println!("{}", output_list::<serde_json::Value>(&[], 0, FILTER, format));'
    # Line i+3: '        } else {'
    # Line i+4: '            print_warning("MESSAGE");'
    # Line i+5: '        }'
    # Line i+6: '        return Ok(());'
    # Line i+7: '    }'

    for i, line in enumerate(lines):
        line_stripped = line.strip()
        # Match: if XXX.is_empty() {
        if not re.match(r'^if\s+\w+\.is_empty\(\)\s*\{\s*$', line_stripped):
            continue
        # Check if we have enough lines
        if i + 7 >= len(lines):
            continue
        # Line i+1: if format.is_json() {
        if not re.match(r'^if\s+\w+\.is_json\(\)\s*\{\s*$', lines[i+1].strip()):
            continue
        # Line i+2: output_list call
        m2 = re.search(r'output_list::<serde_json::Value>\(&\[\],\s*\d+,\s*(.+?),\s*\w+\)', lines[i+2])
        if not m2:
            continue
        # Line i+3: } else {
        if lines[i+3].strip() != '} else {':
            continue
        # Line i+4: print_warning("...");
        m4 = re.search(r'print_warning\("(.+?)"\)', lines[i+4])
        if not m4:
            continue
        # Line i+5: }
        if lines[i+5].strip() != '}':
            continue
        # Line i+6: return Ok(());
        if not re.match(r'^return\s+Ok\(\(\)\);\s*$', lines[i+6].strip()):
            continue
        # Line i+7: }
        if lines[i+7].strip() != '}':
            continue

        # All lines matched! Extract values
        var_match = re.match(r'if\s+(\w+)\.is_empty\(\)', line_stripped)
        var = var_match.group(1)
        filter_expr = m2.group(1).strip()
        msg = m4.group(1)

        # Build indent from original line
        indent_match = re.match(r'^(\s*)', line)
        indent = indent_match.group(1)

        # Determine which macro arm to use
        if msg == "No records found." and filter_expr == "None::<&str>":
            new_line = f"{indent}i_rs_core::handle_empty!({var}, format);\n"
        elif msg == "No records found.":
            new_line = f"{indent}i_rs_core::handle_empty!({var}, format, {filter_expr});\n"
        else:
            new_line = f"{indent}i_rs_core::handle_empty!({var}, format, {filter_expr}, \"{msg}\");\n"

        # Replace the block with the single macro call
        new_lines = lines[:i] + [new_line] + lines[i+8:]
        with open(path, 'w') as f:
            f.writelines(new_lines)

        print(f"OK   {entry}: var={var} filter={filter_expr} msg=\"{msg}\"")
        return True

    print(f"SKIP {entry}: pattern not found")
    return False


for entry in sorted(os.listdir(CRATES_DIR)):
    if entry.startswith("i-rs-"):
        process_crate(entry)
