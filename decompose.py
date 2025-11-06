#!/usr/bin/env python3
"""
Script to decompose main.rs into separate files by function
"""

import re
import os
from pathlib import Path

def extract_functions_and_structures(content):
    """Extract all functions, structs, enums, and other top-level items from Rust code"""

    items = []
    lines = content.split('\n')
    i = 0

    while i < len(lines):
        line = lines[i].strip()

        # Skip empty lines and single-line comments
        if not line or line.startswith('//'):
            i += 1
            continue

        # Check for function definition
        fn_match = re.match(r'^(pub\s+)?(async\s+)?fn\s+(\w+)', line)
        if fn_match:
            func_name = fn_match.group(3)
            start_line = i

            # Collect full function including its body
            brace_count = 0
            function_lines = []
            started = False

            while i < len(lines):
                current_line = lines[i]
                function_lines.append(current_line)

                # Count braces to determine when function ends
                for char in current_line:
                    if char == '{':
                        brace_count += 1
                        started = True
                    elif char == '}':
                        brace_count -= 1

                i += 1

                # Function ends when braces are balanced
                if started and brace_count == 0:
                    break

            items.append({
                'type': 'function',
                'name': func_name,
                'content': '\n'.join(function_lines),
                'start': start_line,
                'end': i
            })
            continue

        # Check for struct definition
        struct_match = re.match(r'^(#\[.*\]\s*)?(pub\s+)?struct\s+(\w+)', line)
        if struct_match:
            struct_name = struct_match.group(3)
            start_line = i

            # Collect struct with attributes
            struct_lines = []

            # Go back to collect attributes
            j = i - 1
            while j >= 0 and (lines[j].strip().startswith('#[') or lines[j].strip() == ''):
                if lines[j].strip().startswith('#['):
                    struct_lines.insert(0, lines[j])
                j -= 1

            # Collect struct body
            brace_count = 0
            started = False

            while i < len(lines):
                current_line = lines[i]
                struct_lines.append(current_line)

                for char in current_line:
                    if char == '{':
                        brace_count += 1
                        started = True
                    elif char == '}':
                        brace_count -= 1

                i += 1

                # Struct ends when braces are balanced or it's a simple struct
                if started and brace_count == 0:
                    break
                elif not started and current_line.strip().endswith(';'):
                    break

            items.append({
                'type': 'struct',
                'name': struct_name,
                'content': '\n'.join(struct_lines),
                'start': start_line,
                'end': i
            })
            continue

        # Check for enum definition
        enum_match = re.match(r'^(#\[.*\]\s*)?(pub\s+)?enum\s+(\w+)', line)
        if enum_match:
            enum_name = enum_match.group(3)
            start_line = i

            enum_lines = []

            # Go back to collect attributes
            j = i - 1
            while j >= 0 and (lines[j].strip().startswith('#[') or lines[j].strip() == ''):
                if lines[j].strip().startswith('#['):
                    enum_lines.insert(0, lines[j])
                j -= 1

            brace_count = 0
            started = False

            while i < len(lines):
                current_line = lines[i]
                enum_lines.append(current_line)

                for char in current_line:
                    if char == '{':
                        brace_count += 1
                        started = True
                    elif char == '}':
                        brace_count -= 1

                i += 1

                if started and brace_count == 0:
                    break

            items.append({
                'type': 'enum',
                'name': enum_name,
                'content': '\n'.join(enum_lines),
                'start': start_line,
                'end': i
            })
            continue

        # Check for impl block
        impl_match = re.match(r'^impl\s+(\w+)', line)
        if impl_match:
            impl_name = impl_match.group(1)
            start_line = i

            impl_lines = []
            brace_count = 0
            started = False

            while i < len(lines):
                current_line = lines[i]
                impl_lines.append(current_line)

                for char in current_line:
                    if char == '{':
                        brace_count += 1
                        started = True
                    elif char == '}':
                        brace_count -= 1

                i += 1

                if started and brace_count == 0:
                    break

            items.append({
                'type': 'impl',
                'name': f'impl_{impl_name}',
                'content': '\n'.join(impl_lines),
                'start': start_line,
                'end': i
            })
            continue

        i += 1

    return items

def extract_imports_and_constants(content):
    """Extract imports, constants, and other top-level declarations"""
    imports = []
    constants = []
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Extract use statements
        if line.startswith('use '):
            imports.append(lines[i])

        # Extract const statements
        elif line.startswith('const '):
            const_lines = [lines[i]]
            i += 1
            # Handle multi-line constants
            while i < len(lines) and not lines[i].strip().endswith(';'):
                const_lines.append(lines[i])
                i += 1
            if i < len(lines):
                const_lines.append(lines[i])
            constants.append('\n'.join(const_lines))
            continue

        i += 1

    return imports, constants

def main():
    main_rs_path = '/home/user/aicommit/src/main.rs'
    decomp_dir = '/home/user/aicommit/decomposition'

    # Read main.rs
    with open(main_rs_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Extract imports and constants
    imports, constants = extract_imports_and_constants(content)

    # Save imports
    with open(f'{decomp_dir}/00_imports.rs', 'w', encoding='utf-8') as f:
        f.write('\n'.join(imports))

    # Save constants
    with open(f'{decomp_dir}/01_constants.rs', 'w', encoding='utf-8') as f:
        f.write('\n\n'.join(constants))

    # Extract all items
    items = extract_functions_and_structures(content)

    # Save each item to a separate file
    for idx, item in enumerate(items):
        filename = f"{decomp_dir}/{idx:03d}_{item['type']}_{item['name']}.rs"
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(item['content'])
        print(f"Extracted {item['type']} '{item['name']}' to {filename}")

    print(f"\nTotal items extracted: {len(items)}")
    print(f"Imports saved to: {decomp_dir}/00_imports.rs")
    print(f"Constants saved to: {decomp_dir}/01_constants.rs")

if __name__ == '__main__':
    main()
