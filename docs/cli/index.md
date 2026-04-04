# CLI

The AgilePlus CLI provides command-line access to all project management features.

## Installation

```bash
# Install via cargo
cargo install agileplus

# Or build from source
git clone https://github.com/KooshaPari/agileplus.git
cd agileplus
cargo build --release
```

## Quick Start

```bash
# Initialize a new project
agileplus init my-project

# Create a feature
agileplus feature create --title "New Feature" --project my-project

# List work packages
agileplus wp list --feature <feature-id>

# Update work package status
agileplus wp status <wp-id> --state implementing
```

## Commands

See individual command documentation for detailed usage.
