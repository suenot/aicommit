# Version Management Commands

<cite>
**Referenced Files in This Document **   
- [src/main.rs](file://src/main.rs)
- [Cargo.toml](file://Cargo.toml)
- [package.json](file://package.json)
- [readme.md](file://readme.md)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Version Management Architecture](#version-management-architecture)
3. [Core Versioning Components](#core-versioning-components)
4. [Version Synchronization Workflow](#version-synchronization-workflow)
5. [Command Implementation Details](#command-implementation-details)
6. [Integration with Git Operations](#integration-with-git-operations)
7. [Error Handling and Edge Cases](#error-handling-and-edge-cases)
8. [Best Practices for Release Automation](#best-practices-for-release-automation)

## Introduction
The aicommit tool provides comprehensive version management capabilities that enable synchronized version updates across multiple package managers and repository systems. This documentation details the implementation and usage of version management commands including `--version-bump`, `--version-file`, `--sync-cargo`, and related flags. The system ensures consistent versioning across Cargo.toml, package.json, and GitHub tags through a coordinated workflow that maintains version integrity and prevents drift between different package management systems.

**Section sources**
- [readme.md](file://readme.md#L1-L734)

## Version Management Architecture

```mermaid
graph TD
A[Version Command Execution] --> B{Check version-file}
B --> |Not specified| C[Abort with error]
B --> |Specified| D[Read current version]
D --> E{version-iterate?}
E --> |Yes| F[Increment version number]
E --> |No| G[Use existing version]
F --> H[Update version file]
G --> I{version-cargo?}
H --> I
I --> |Yes| J[Update Cargo.toml & Cargo.lock]
I --> |No| K{version-npm?}
J --> K
K --> |Yes| L[Update package.json]
K --> |No| M{version-github?}
L --> M
M --> |Yes| N[Create Git tag vX.X.X]
M --> |No| O[Complete version update]
N --> P[Push tag to origin]
P --> O
O --> Q[Stage all version changes]
Q --> R[Proceed with commit process]
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L260-L301)
- [src/main.rs](file://src/main.rs#L1844-L1923)

## Core Versioning Components

### Version Increment Logic
The version increment functionality is implemented through the `increment_version` function which parses semantic version strings and increments the patch number. The function handles standard semantic versioning format (MAJOR.MINOR.PATCH) by splitting the version string on periods and incrementing the rightmost numeric component.

```mermaid
flowchart TD
Start([Start increment_version]) --> Parse["Parse version string by '.'"]
Parse --> Validate{"Valid format?"}
Validate --> |No| Error["Return 'Invalid version format'"]
Validate --> |Yes| Extract["Extract last component"]
Extract --> Convert{"Numeric value?"}
Convert --> |No| Error
Convert --> |Yes| Increment["Increment by 1"]
Increment --> Rejoin["Rejoin components with '.'"]
Rejoin --> Return["Return new version string"]
Return --> End([End])
Error --> End
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L260-L301)

### Configuration File Updates
The system implements separate functions for updating different configuration files, ensuring proper formatting and syntax preservation:

```mermaid
classDiagram
class VersionManager {
+increment_version(version : &str) Result~String~
+update_version_file(file_path : &str) Result~()~
+update_cargo_version(version : &str) Result~()~
+update_npm_version(version : &str) Result~()~
+update_github_version(version : &str) Result~()~
}
class CargoUpdater {
-update_cargo_toml(version : &str)
-update_cargo_lock(version : &str)
-run_cargo_update()
}
class NPMUpdater {
-parse_package_json()
-update_version_field(version : &str)
-write_formatted_json()
}
class GitHubTagger {
-check_tag_exists(version : &str)
-create_annotated_tag(version : &str)
-push_tag_to_origin(version : &str)
}
VersionManager --> CargoUpdater : "delegates"
VersionManager --> NPMUpdater : "delegates"
VersionManager --> GitHubTagger : "delegates"
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L303-L343)
- [src/main.rs](file://src/main.rs#L345-L385)

**Section sources**
- [src/main.rs](file://src/main.rs#L260-L301)
- [src/main.rs](file://src/main.rs#L303-L385)

## Version Synchronization Workflow

```mermaid
sequenceDiagram
participant CLI as Command Line
participant VM as VersionManager
participant FS as File System
participant Git as Git Repository
CLI->>VM : Execute --version-iterate --version-cargo --version-npm --version-github
VM->>FS : Read version file content
FS-->>VM : Return current version
VM->>VM : Increment version number
VM->>FS : Write updated version to file
VM->>FS : Update Cargo.toml with new version
VM->>FS : Update Cargo.lock with new version
VM->>FS : Run cargo update command
VM->>FS : Update package.json with new version
VM->>Git : Check if v(new_version) tag exists
Git-->>VM : Return tag status
alt Tag doesn't exist
VM->>Git : Create annotated tag v(new_version)
VM->>Git : Push tag to origin
end
VM->>Git : git add . to stage all changes
VM-->>CLI : Confirm version synchronization complete
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L1844-L1923)
- [src/main.rs](file://src/main.rs#L303-L385)

## Command Implementation Details

### Command Line Interface Definitions
The version management commands are defined in the CLI structure with specific parameters for controlling version synchronization behavior:

```mermaid
erDiagram
CLI_COMMANDS {
boolean version_iterate PK
string version_file FK
boolean version_cargo
boolean version_npm
boolean version_github
}
VERSION_FILE {
string path PK
string content
datetime modified_at
}
CARGO_TOML {
string version PK
string name
string edition
}
PACKAGE_JSON {
string version PK
string name
string description
}
GITHUB_TAG {
string tag_name PK
string message
datetime created_at
}
CLI_COMMANDS ||--o{ VERSION_FILE : "reads/writes"
CLI_COMMANDS }|--o{ CARGO_TOML : "updates"
CLI_COMMANDS }|--o{ PACKAGE_JSON : "updates"
CLI_COMMANDS }|--o{ GITHUB_TAG : "creates"
```

**Section sources**
- [src/main.rs](file://src/main.rs#L146-L175)

### Version Update Dependencies
The implementation enforces strict dependencies between version management commands, requiring a version file as the source of truth for all synchronization operations:

```mermaid
flowchart LR
A[version-file] --> B{Required for}
B --> C[version-iterate]
B --> D[version-cargo]
B --> E[version-npm]
B --> F[version-github]
style A fill:#f9f,stroke:#333
style B fill:#bbf,stroke:#333,color:#fff
style C fill:#9f9,stroke:#333
style D fill:#9f9,stroke:#333
style E fill:#9f9,stroke:#333
style F fill:#9f9,stroke:#333
classDef required fill:#f9f,stroke:#333;
classDef dependent fill:#9f9,stroke:#333;
class A required
class C,D,E,F dependent
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L1844-L1885)
- [src/main.rs](file://src/main.rs#L1887-L1892)

## Integration with Git Operations
The version management system integrates seamlessly with Git operations, automatically staging version changes and supporting tag creation and push operations:

```mermaid
flowchart TD
A[Version Commands Executed] --> B[Stage version changes]
B --> C{git add .}
C --> D[Get git diff]
D --> E[Generate commit message]
E --> F{pull requested?}
F --> |Yes| G[git pull --no-rebase]
F --> |No| H{push requested?}
G --> H
H --> |Yes| I[git push with upstream setup]
H --> |No| J[Commit complete]
I --> J
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L1887-L1923)
- [src/main.rs](file://src/main.rs#L1925-L2149)

**Section sources**
- [src/main.rs](file://src/main.rs#L1844-L2149)

## Error Handling and Edge Cases
The system implements comprehensive error handling for various edge cases that may occur during version management operations:

```mermaid
stateDiagram-v2
[*] --> Initial
Initial --> ReadVersionFile : version-file specified
ReadVersionFile --> IncrementVersion : success
ReadVersionFile --> ErrorState : file not found
IncrementVersion --> ValidateFormat : parse version
ValidateFormat --> ErrorState : invalid format
ValidateFormat --> UpdateFiles : valid version
UpdateFiles --> UpdateCargo : version-cargo flag
UpdateFiles --> UpdateNPM : version-npm flag
UpdateFiles --> CreateTag : version-github flag
UpdateCargo --> ErrorState : Cargo.toml not found
UpdateNPM --> ErrorState : package.json not found
CreateTag --> CheckExisting : tag already exists?
CheckExisting --> SuccessState : tag created
CheckExisting --> SuccessState : tag exists (no-op)
ErrorState --> [*] : return error
SuccessState --> StageChanges : git add .
StageChanges --> [*] : continue commit
```

**Diagram sources **
- [src/main.rs](file://src/main.rs#L260-L301)
- [src/main.rs](file://src/main.rs#L1844-L1885)

**Section sources**
- [src/main.rs](file://src/main.rs#L260-L301)
- [src/main.rs](file://src/main.rs#L1844-L1885)

## Best Practices for Release Automation
The aicommit tool supports automated release workflows through scriptable commands that can be integrated into CI/CD pipelines. The recommended approach combines version bumping with automatic pushing to ensure consistent releases:

```mermaid
flowchart TB
A[Development Complete] --> B[Run release script]
B --> C[aicommit --add]
C --> D[--version-file version]
D --> E[--version-iterate]
E --> F[--version-cargo]
F --> G[--version-npm]
G --> H[--version-github]
H --> I[--push]
I --> J[Automatic release created]
J --> K[New version tagged]
K --> L[Changes pushed to origin]
style A fill:#f96,stroke:#333
style J fill:#6f9,stroke:#333
style K fill:#6f9,stroke:#333
style L fill:#6f9,stroke:#333
```

**Section sources**
- [package.json](file://package.json#L45-L47)
- [readme.md](file://readme.md#L1-L734)