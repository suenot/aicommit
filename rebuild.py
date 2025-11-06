#!/usr/bin/env python3
"""
Script to rebuild main.rs from decomposed files into modular structure
"""

import os
import glob

def read_file(path):
    """Read file content"""
    with open(path, 'r', encoding='utf-8') as f:
        return f.read()

def write_file(path, content):
    """Write content to file"""
    with open(path, 'w', encoding='utf-8') as f:
        f.write(content)

def main():
    decomp_dir = '/home/user/aicommit/decomposition'
    src_dir = '/home/user/aicommit/src'

    # Read all decomposed files
    files = sorted(glob.glob(f'{decomp_dir}/*.rs'))

    # Categorize files
    imports = []
    constants = []
    structs = []
    enums = []
    impls = []
    functions = []

    for file_path in files:
        filename = os.path.basename(file_path)
        content = read_file(file_path)

        if '00_imports' in filename:
            imports.append(content)
        elif '01_constants' in filename:
            constants.append(content)
        elif '_struct_' in filename:
            structs.append((filename, content))
        elif '_enum_' in filename:
            enums.append((filename, content))
        elif '_impl_' in filename:
            impls.append((filename, content))
        elif '_function_' in filename:
            functions.append((filename, content))

    # Group functions by category
    version_functions = []
    git_functions = []
    provider_functions = []
    model_functions = []
    main_functions = []
    other_functions = []

    for filename, content in functions:
        if 'version' in filename.lower():
            version_functions.append((filename, content))
        elif 'git' in filename.lower() or 'commit' in filename.lower():
            git_functions.append((filename, content))
        elif 'openrouter' in filename.lower() or 'ollama' in filename.lower() or \
             'openai' in filename.lower() or 'provider' in filename.lower() or \
             'claude' in filename.lower() or 'opencode' in filename.lower():
            provider_functions.append((filename, content))
        elif 'model' in filename.lower():
            model_functions.append((filename, content))
        elif 'main' in filename.lower() or 'watch' in filename.lower() or 'dry_run' in filename.lower():
            main_functions.append((filename, content))
        else:
            other_functions.append((filename, content))

    # Create types.rs module (structs, enums, impls)
    types_content = "// Types module - structures, enums, and implementations\n\n"
    types_content += "use serde::{Serialize, Deserialize};\n"
    types_content += "use clap::Parser;\n"
    types_content += "use chrono;\n\n"

    # Add structs
    for filename, content in structs:
        types_content += f"// From: {filename}\n"
        types_content += content + "\n\n"

    # Add enums
    for filename, content in enums:
        types_content += f"// From: {filename}\n"
        types_content += content + "\n\n"

    # Add impls
    for filename, content in impls:
        types_content += f"// From: {filename}\n"
        types_content += content + "\n\n"

    write_file(f'{src_dir}/types.rs', types_content)
    print(f"Created types.rs ({len(structs)} structs, {len(enums)} enums, {len(impls)} impls)")

    # Create version.rs module
    version_content = "// Version management functions\n\n"
    version_content += "use std::process::Command;\n"
    version_content += "use serde_json::json;\n"
    version_content += "use tracing::{info, error, debug};\n\n"

    for filename, content in version_functions:
        version_content += f"// From: {filename}\n"
        version_content += content + "\n\n"

    write_file(f'{src_dir}/version.rs', version_content)
    print(f"Created version.rs ({len(version_functions)} functions)")

    # Create git.rs module
    git_content = "// Git operations\n\n"
    git_content += "use std::process::Command;\n"
    git_content += "use dialoguer::Input;\n"
    git_content += "use tracing::{info, error, debug};\n\n"

    for filename, content in git_functions:
        git_content += f"// From: {filename}\n"
        git_content += content + "\n\n"

    write_file(f'{src_dir}/git.rs', git_content)
    print(f"Created git.rs ({len(git_functions)} functions)")

    # Create providers.rs module
    providers_content = "// AI provider functions\n\n"
    providers_content += "use crate::types::*;\n"
    providers_content += "use dialoguer::{Input, Select};\n"
    providers_content += "use uuid::Uuid;\n"
    providers_content += "use std::fs;\n"
    providers_content += "use serde_json;\n"
    providers_content += "use tracing::{info, error, debug};\n\n"

    for filename, content in provider_functions:
        providers_content += f"// From: {filename}\n"
        providers_content += content + "\n\n"

    write_file(f'{src_dir}/providers.rs', providers_content)
    print(f"Created providers.rs ({len(provider_functions)} functions)")

    # Create models.rs module
    models_content = "// Model management functions\n\n"
    models_content += "use crate::types::*;\n"
    models_content += "use std::fs;\n"
    models_content += "use chrono;\n"
    models_content += "use tracing::{info, error, debug};\n\n"

    for filename, content in model_functions:
        models_content += f"// From: {filename}\n"
        models_content += content + "\n\n"

    write_file(f'{src_dir}/models.rs', models_content)
    print(f"Created models.rs ({len(model_functions)} functions)")

    # Create utils.rs for other functions
    utils_content = "// Utility functions\n\n"
    utils_content += "use std::time::Duration;\n"
    utils_content += "use tracing::{info, error, debug};\n\n"

    for filename, content in other_functions:
        utils_content += f"// From: {filename}\n"
        utils_content += content + "\n\n"

    write_file(f'{src_dir}/utils.rs', utils_content)
    print(f"Created utils.rs ({len(other_functions)} functions)")

    # Create new main.rs
    main_content = "// Main module - orchestrates all functionality\n\n"

    # Add imports
    if imports:
        main_content += imports[0] + "\n\n"

    # Add module declarations
    main_content += "// Module declarations\n"
    main_content += "mod logging;\n"
    main_content += "mod types;\n"
    main_content += "mod version;\n"
    main_content += "mod git;\n"
    main_content += "mod providers;\n"
    main_content += "mod models;\n"
    main_content += "mod utils;\n\n"

    # Add use statements for our modules
    main_content += "// Use declarations from our modules\n"
    main_content += "use types::*;\n"
    main_content += "use version::*;\n"
    main_content += "use git::*;\n"
    main_content += "use providers::*;\n"
    main_content += "use models::*;\n"
    main_content += "use utils::*;\n"
    main_content += "use logging::{LoggingConfig, init_logging, init_default_logging, log_error, log_info, log_warning};\n\n"

    # Add constants
    if constants:
        main_content += "// Constants\n"
        main_content += constants[0] + "\n\n"

    # Add main functions
    for filename, content in main_functions:
        main_content += f"// From: {filename}\n"
        main_content += content + "\n\n"

    write_file(f'{src_dir}/main.rs', main_content)
    print(f"Created main.rs ({len(main_functions)} functions)")

    # Count lines in new main.rs
    lines = len(read_file(f'{src_dir}/main.rs').split('\n'))
    print(f"\nNew main.rs has {lines} lines")

    print("\nModular structure created successfully!")
    print("- types.rs: Data structures and implementations")
    print("- version.rs: Version management")
    print("- git.rs: Git operations")
    print("- providers.rs: AI provider integrations")
    print("- models.rs: Model management")
    print("- utils.rs: Utility functions")
    print("- main.rs: Main application logic")

if __name__ == '__main__':
    main()
