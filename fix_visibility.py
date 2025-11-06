#!/usr/bin/env python3
"""
Script to add pub modifiers to all top-level items in modules
"""

import re
import os

def fix_visibility(content):
    """Add pub to functions, structs, enums, and const declarations"""
    lines = content.split('\n')
    result = []

    for line in lines:
        stripped = line.strip()

        # Skip comments, empty lines, and already public items
        if not stripped or stripped.startswith('//') or stripped.startswith('use ') or stripped.startswith('pub '):
            result.append(line)
            continue

        # Add pub to struct definitions
        if re.match(r'^struct\s+\w+', stripped):
            line = line.replace('struct ', 'pub struct ', 1)

        # Add pub to enum definitions
        elif re.match(r'^enum\s+\w+', stripped):
            line = line.replace('enum ', 'pub enum ', 1)

        # Add pub to function definitions
        elif re.match(r'^(async\s+)?fn\s+\w+', stripped):
            # Check if async
            if stripped.startswith('async '):
                line = line.replace('async fn ', 'pub async fn ', 1)
            else:
                line = line.replace('fn ', 'pub fn ', 1)

        # Add pub to const definitions
        elif re.match(r'^const\s+\w+', stripped):
            line = line.replace('const ', 'pub const ', 1)

        # Add pub to impl blocks
        elif re.match(r'^impl\s+', stripped):
            line = line.replace('impl ', 'pub impl ', 1)

        result.append(line)

    return '\n'.join(result)

def process_file(filepath):
    """Process a single file"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    fixed_content = fix_visibility(content)

    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(fixed_content)

    print(f"Fixed visibility in {filepath}")

def main():
    src_dir = '/home/user/aicommit/src'

    # Process all module files except main.rs and logging.rs
    modules = ['types.rs', 'version.rs', 'git.rs', 'providers.rs', 'models.rs', 'utils.rs']

    for module in modules:
        filepath = os.path.join(src_dir, module)
        if os.path.exists(filepath):
            process_file(filepath)

    # Fix main.rs - remove duplicate logging import
    main_path = os.path.join(src_dir, 'main.rs')
    with open(main_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Remove duplicate logging import
    lines = content.split('\n')
    seen_logging = False
    fixed_lines = []

    for line in lines:
        if 'use logging::' in line:
            if not seen_logging:
                seen_logging = True
                fixed_lines.append(line)
        else:
            fixed_lines.append(line)

    with open(main_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(fixed_lines))

    print("Fixed main.rs")
    print("\nAll visibility modifiers fixed!")

if __name__ == '__main__':
    main()
