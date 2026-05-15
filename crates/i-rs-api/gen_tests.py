#!/usr/bin/env python3
"""
Generate #[cfg(test)] modules for all 70 CLI tool crates.

For each crate, reads its main.rs Commands enum, extracts field types,
and generates src/tests.rs with CRUD smoke tests using a temp CONFIG_DIR.
"""

import os
import re

CRATES_DIR = os.path.join(os.path.dirname(__file__), '..')
SKIP = {'i-rs', 'i-rs-core', 'i-rs-api'}

# Type-to-test-value mapping for Commands enum fields
TYPE_VALUES = {
    'String': '"test-value".to_string()',
    'f64': '1.0',
    'i32': '1',
    'u32': '1',
    'i64': '1',
    'u16': '8080',
    'f32': '1.0',
}


def _extract_fields(body):
    """Extract (name, type) pairs from a variant's brace-enclosed body."""
    fields = []
    for line in body.split('\n'):
        line = line.strip()
        if not line or line.startswith('#') or line.startswith('//'):
            continue
        fm = re.match(r'(?:#\[.*?\]\s*)*(?:pub\s+)?(\w+)\s*:\s*([^,]+)', line)
        if fm:
            fields.append((fm.group(1), fm.group(2).strip()))
    return fields


def parse_main_rs(path):
    """Parse a crate's main.rs to extract Commands enum structure."""
    with open(path) as f:
        content = f.read()

    result = {
        'has_add': False, 'has_update': False, 'has_delete': False,
        'has_get': False, 'has_list': False, 'has_example': False,
        'has_skill': False, 'has_data': False, 'has_done': False,
        'add_fields': [], 'get_fields': [], 'delete_fields': [],
        'update_fields': [],
        'get_key_field': None, 'get_key_type': None,
        'delete_key_field': None, 'delete_key_type': None,
        'update_key_field': None, 'update_key_type': None,
        'primary_key_field': None,
        'is_subcmd_style': False, 'is_tuple_style': False,
        'has_uuid_key': False, 'has_custom_types': False,
        'uses_keychain': False,
        'run_uses_json_bool': False,
        'output_format_path': 'i_rs_core::presentation::OutputFormat',
    }

    if 'use i_rs_core::presentation::OutputFormat' in content:
        result['output_format_path'] = 'i_rs_core::presentation::OutputFormat'
    else:
        result['output_format_path'] = 'crate::presentation::OutputFormat'

    if 'init_keyring' in content or 'keyring::' in content:
        result['uses_keychain'] = True

    if 'fn run(command: Commands, json: bool)' in content:
        result['run_uses_json_bool'] = True

    if 'Subcommands' in content and 'GetSubcommands' in content:
        result['is_subcmd_style'] = True

    m = re.search(r'enum Commands\s*\{', content)
    if not m:
        return result

    enum_start = m.end()
    depth = 1
    i = enum_start
    while i < len(content) and depth > 0:
        if content[i] == '{':
            depth += 1
        elif content[i] == '}':
            depth -= 1
        i += 1
    enum_body = content[enum_start:i - 1]

    result['has_add'] = bool(re.search(r'\bAdd\s*\{', enum_body))
    result['has_update'] = bool(re.search(r'\bUpdate\s*\{', enum_body))
    result['has_delete'] = bool(re.search(r'\bDelete\s*\{', enum_body))
    result['has_get'] = bool(re.search(r'\bGet\s*\{', enum_body))
    result['has_list'] = bool(re.search(r'\bList\s*\{', enum_body))
    result['has_example'] = bool(re.search(r'\bExample', enum_body))
    result['has_skill'] = bool(re.search(r'\bSkill', enum_body))
    result['has_data'] = bool(re.search(r'\bData', enum_body))
    result['has_done'] = bool(re.search(r'\bDone', enum_body))

    result['is_tuple_style'] = bool(
        re.search(r'\b(?:Add|Get|Delete|Update)\s*\(', enum_body)
    )

    for variant_name in ['Add', 'Get', 'Delete', 'Update']:
        key = variant_name.lower()
        if re.search(rf'\b{variant_name}\s*\(', enum_body):
            continue
        m = re.search(rf'{variant_name}\s*\{{(.*?)\}}', enum_body, re.DOTALL)
        if m:
            fields = _extract_fields(m.group(1))
            result[f'{key}_fields'] = fields

    add_field_names = {name for name, _ in result['add_fields']}

    known_scalars = {'String', 'f64', 'i32', 'u32', 'i64', 'u16', 'f32', 'bool', 'chrono::NaiveDate'}
    pk = None
    for name, typ in result['add_fields']:
        if typ == 'String' or typ.startswith('String'):
            pk = name
            break
    if not pk and result['add_fields']:
        for name, typ in result['add_fields']:
            if not typ.startswith('Option<') and not typ.startswith('Vec<'):
                pk = name
                break
        if not pk:
            pk = result['add_fields'][0][0]
    result['primary_key_field'] = pk

    for _, typ in result['add_fields']:
        if (typ not in known_scalars
                and not typ.startswith('Option<')
                and not typ.startswith('Vec<')):
            result['has_custom_types'] = True
            break

    for variant_name in ['Get', 'Delete', 'Update']:
        key = variant_name.lower()
        fields = result[f'{key}_fields']
        if not fields:
            continue
        key_field = None
        key_type = None
        for name, typ in fields:
            if typ == 'String':
                key_field = name
                key_type = 'String'
                if name not in add_field_names:
                    result['has_uuid_key'] = True
                break
        if not key_field:
            for name, typ in fields:
                if typ == 'Option<String>':
                    key_field = name
                    key_type = 'Option<String>'
                    break
        if key_field:
            result[f'{key}_key_field'] = key_field
            result[f'{key}_key_type'] = key_type

    return result


def gen_value(typ, field_name='', is_primary_key=False, crate_name='', key_type=None, unique_suffix='', variant_name=''):
    """Generate a Rust expression for a given type."""
    is_date_field = ('date' in field_name.lower() or 'time' in field_name.lower()
                     or field_name in ('bedtime', 'wake_time', 'purchase_date'))
    is_url_field = field_name == 'url'
    if field_name in ('bedtime', 'wake_time'):
        if typ == 'Option<String>':
            return 'Some("22:00".to_string())'
        return '"22:00".to_string()'
    if is_primary_key:
        if typ == 'String':
            if is_date_field:
                if field_name == 'birth_date':
                    return '"01-15".to_string()'
                if unique_suffix and unique_suffix != '1':
                    return '"2024-01-16".to_string()'
                return '"2024-01-15".to_string()'
            if is_url_field:
                return '"https://example.com".to_string()'
            return f'"test-{crate_name}-{unique_suffix}".to_string()'
        if typ == 'Option<String>':
            if is_date_field:
                if field_name == 'birth_date':
                    return 'Some("01-15".to_string())'
                if unique_suffix and unique_suffix != '1':
                    return 'Some("2024-01-16".to_string())'
                return 'Some("2024-01-15".to_string())'
            if is_url_field:
                return 'Some("https://example.com".to_string())'
            return f'Some("test-{crate_name}-{unique_suffix}".to_string())'
    if typ == 'String':
        if field_name in ('bedtime', 'wake_time'):
            return '"22:00".to_string()'
        if field_name == 'gift_type':
            return '"sent".to_string()'
        if is_date_field:
            if field_name == 'birth_date':
                return '"01-15".to_string()'
            return '"2024-01-15".to_string()'
        if is_url_field:
            return '"https://example.com".to_string()'
        return f'"test-{field_name}".to_string()'
    if typ in TYPE_VALUES:
        return TYPE_VALUES[typ]
    if typ.startswith('Vec<'):
        return 'vec![]'
    if field_name in ('left_sphere', 'right_sphere'):
        return 'Some(1.0)'
    if typ.startswith('Option<'):
        return 'None'
    if typ == 'bool':
        if variant_name == 'Delete' and field_name == 'force':
            return 'true'
        return 'false'
    if typ == 'chrono::NaiveDate':
        return 'chrono::Utc::now().date_naive()'
    if '::' in typ:
        segments = typ.rsplit('::', 1)
        return f'{typ}::Stock'
    return 'Default::default()'


def gen_fields_lines(fields, primary_key, crate_name, key_type=None, unique_suffix='', variant_name=''):
    """Generate 'name: value,' lines for all fields in a variant."""
    result_lines = []
    for name, typ in fields:
        is_pk = (name == primary_key)
        kt = key_type if is_pk else None
        val = gen_value(typ, name, is_pk, crate_name, kt, unique_suffix, variant_name)
        result_lines.append(f'            {name}: {val},')
    return result_lines


def gen_tests_module(crate_name, info):
    """Generate the #[cfg(test)] mod tests block for a crate."""
    lines = []
    lines.append('#[cfg(test)]')
    lines.append('mod tests {')
    lines.append('    use crate::{Cli, Commands, commands, run};')
    lines.append('    use clap::Parser;')
    lines.append('')
    lines.append('    fn setup() {')
    lines.append('        use std::sync::OnceLock;')
    lines.append('        static INIT: OnceLock<()> = OnceLock::new();')
    lines.append('        INIT.get_or_init(|| {')
    lines.append('            let tmp = std::env::temp_dir()')
    lines.append(f'                .join(format!("{crate_name}-test-{{}}", std::process::id()));')
    lines.append('            // ensure dir exists')
    lines.append('            let _ = std::fs::create_dir_all(&tmp);')
    lines.append('            unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }')
    lines.append('        });')
    lines.append('    }')
    lines.append('')

    test_idx = 0

    # --- test_help ---
    lines.append('    #[test]')
    lines.append(f'    fn test_help() {{')
    lines.append(f'        let _ = Cli::try_parse_from(["{crate_name}", "--help"]);')
    lines.append('    }')
    lines.append('')

    # --- test_example ---
    if info['has_example'] and not info['is_tuple_style']:
        lines.append('    #[test]')
        lines.append('    fn test_example() {')
        lines.append('        setup();')
        lines.append('        let cmd = Commands::Example {};')
        if crate_name in ('i-rs-plant',):
            lines[-1] = '        let cmd = Commands::Example;'
        lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')
        lines.append('    }')
        lines.append('')

    can_do_crud = not info['is_tuple_style'] and not info['is_subcmd_style'] and not info['has_custom_types'] and not info['uses_keychain']
    has_full_crud = (info['has_add'] and info['add_fields'] and info['has_get']
                     and can_do_crud and not info['has_uuid_key'])
    has_add_only = (info['has_add'] and info['add_fields'] and can_do_crud
                    and not has_full_crud)

    # --- test_crud (single sequential test: add + list + add2 + get + update + delete) ---
    if has_full_crud:
        test_idx += 1
        suffix = str(test_idx)
        lines.append('    #[test]')
        lines.append('    fn test_crud() {')
        lines.append('        setup();')

        # Add first item
        lines.append('        let cmd = Commands::Add {')
        lines.extend(gen_fields_lines(
            info['add_fields'], info['primary_key_field'], crate_name,
            unique_suffix=suffix,
        ))
        lines.append('        };')
        lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')

        # List
        if info['has_list']:
            lines.append(f'        let cmd = Cli::try_parse_from(["{crate_name}", "list"]).unwrap().command;')
            lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')

        # Add second item (for get+update+delete, so first item's list above works on clean data)
        lines.append('        let cmd = Commands::Add {')
        lines.extend(gen_fields_lines(
            info['add_fields'], info['primary_key_field'], crate_name,
            unique_suffix=suffix + 'b', variant_name='Add',
        ))
        lines.append('        };')
        lines.append('        run(cmd, OutputFormat::Table).unwrap();')

        # Get
        if info['get_fields']:
            get_pk = info['get_key_field'] or info['primary_key_field'] or 'name'
            get_kt = info['get_key_type']
            lines.append(f'        let get_cmd = Commands::Get {{')
            lines.extend(gen_fields_lines(
                info['get_fields'], get_pk, crate_name, get_kt,
                unique_suffix=suffix + 'b', variant_name='Get',
            ))
            lines.append('        };')
        else:
            get_pk = info['get_key_field'] or info['primary_key_field'] or 'name'
            pk_val = f'"test-{crate_name}-{suffix}b".to_string()'
            lines.append(f'        let get_cmd = Commands::Get {{ {get_pk}: {pk_val} }};')
        lines.append('        assert!(run(get_cmd, OutputFormat::Table).is_ok());')

        # Update
        if info['has_update'] and info['update_fields']:
            upd_pk = info['update_key_field'] or info['primary_key_field'] or 'name'
            upd_kt = info['update_key_type']
            lines.append('        let update_cmd = Commands::Update {')
            lines.extend(gen_fields_lines(
                info['update_fields'], upd_pk, crate_name, upd_kt,
                unique_suffix=suffix + 'b', variant_name='Update',
            ))
            lines.append('        };')
            lines.append('        assert!(run(update_cmd, OutputFormat::Table).is_ok());')

        # Delete
        if info['has_delete'] and info['delete_fields']:
            del_pk = info['delete_key_field'] or info['primary_key_field'] or 'name'
            del_kt = info['delete_key_type']
            lines.append('        let del_cmd = Commands::Delete {')
            lines.extend(gen_fields_lines(
                info['delete_fields'], del_pk, crate_name, del_kt,
                unique_suffix=suffix + 'b', variant_name='Delete',
            ))
            lines.append('        };')
            lines.append('        assert!(run(del_cmd, OutputFormat::Table).is_ok());')

        lines.append('    }')
        lines.append('')

    # --- test_add_and_list (for crates without full CRUD, e.g. UUID-keyed) ---
    if has_add_only:
        test_idx += 1
        suffix = str(test_idx)
        lines.append('    #[test]')
        lines.append('    fn test_add_and_list() {')
        lines.append('        setup();')
        lines.append('        let cmd = Commands::Add {')
        lines.extend(gen_fields_lines(
            info['add_fields'], info['primary_key_field'], crate_name,
            unique_suffix=suffix, variant_name='Add',
        ))
        lines.append('        };')
        lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')
        if info['has_list']:
            lines.append(f'        let cmd = Cli::try_parse_from(["{crate_name}", "list"]).unwrap().command;')
            lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')
        lines.append('    }')
        lines.append('')

    # --- test_get_not_found ---
    if info['has_get'] and can_do_crud:
        pk = info['get_key_field'] or info['primary_key_field'] or 'name'
        if info['has_uuid_key']:
            not_found_val = '"00000000-0000-0000-0000-000000000000".to_string()'
        else:
            not_found_val = '"nonexistent".to_string()'
        if info['get_key_type'] == 'Option<String>':
            not_found_val = f'Some({not_found_val})'
        lines.append('    #[test]')
        lines.append('    fn test_get_not_found() {')
        lines.append('        setup();')
        if info['get_fields']:
            lines.append('        let get_cmd = Commands::Get {')
            for name, typ in info['get_fields']:
                if name == pk:
                    lines.append(f'            {name}: {not_found_val},')
                else:
                    val = gen_value(typ, name, False, crate_name, None)
                    lines.append(f'            {name}: {val},')
            lines.append('        };')
        else:
            lines.append(f'        let get_cmd = Commands::Get {{ {pk}: {not_found_val} }};')
        lines.append('        assert!(run(get_cmd, OutputFormat::Table).is_err());')
        lines.append('    }')
        lines.append('')

    # --- test_data_export ---
    if info['has_data']:
        lines.append('    #[test]')
        lines.append('    fn test_data_export() {')
        lines.append('        setup();')
        lines.append('        let cmd = Commands::Data(commands::data::DataCommand::Export);')
        lines.append('        assert!(run(cmd, OutputFormat::Table).is_ok());')
        lines.append('    }')
        lines.append('')

    lines.append('}')
    test_code = '\n'.join(lines)
    test_code = test_code.replace('OutputFormat::Table', f'{info["output_format_path"]}::Table')
    test_code = test_code.replace('OutputFormat::Json', f'{info["output_format_path"]}::Json')
    if info['run_uses_json_bool']:
        test_code = test_code.replace(f'{info["output_format_path"]}::Table', 'false')
        test_code = test_code.replace(f'{info["output_format_path"]}::Json', 'true')
    return test_code


def main():
    crates = sorted(os.listdir(CRATES_DIR))
    for d in crates:
        if not d.startswith('i-rs-') or d in SKIP:
            continue
        main_path = os.path.join(CRATES_DIR, d, 'src', 'main.rs')
        tests_path = os.path.join(CRATES_DIR, d, 'src', 'tests.rs')
        if not os.path.exists(main_path):
            print(f"SKIP {d}: no main.rs")
            continue

        info = parse_main_rs(main_path)
        test_code = gen_tests_module(d, info)
        with open(tests_path, 'w') as f:
            f.write(test_code)
        print(f"GEN  {d}: add={info['has_add']} update={info['has_update']} "
              f"delete={info['has_delete']} get={info['has_get']} "
              f"pk={info['primary_key_field']} subcmd={info['is_subcmd_style']} "
              f"tuple={info['is_tuple_style']}")

    print("\n=== Adding mod tests; to main.rs files ===")
    for d in crates:
        if not d.startswith('i-rs-') or d in SKIP:
            continue
        main_path = os.path.join(CRATES_DIR, d, 'src', 'main.rs')
        if not os.path.exists(main_path):
            continue
        with open(main_path) as f:
            content = f.read()
        if 'mod tests;' in content or '#[cfg(test)]' in content:
            continue
        lines = content.split('\n')
        insert_pos = len(lines)
        for i in range(len(lines) - 1, -1, -1):
            stripped = lines[i].strip()
            if stripped and not stripped.startswith('//'):
                insert_pos = i + 1
                break
        lines.insert(insert_pos, '')
        lines.insert(insert_pos + 1, '#[cfg(test)]')
        lines.insert(insert_pos + 2, 'mod tests;')
        with open(main_path, 'w') as f:
            f.write('\n'.join(lines))
        print(f"  + mod tests; in {d}/src/main.rs")


if __name__ == '__main__':
    main()
