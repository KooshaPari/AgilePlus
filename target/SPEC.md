# Specification: target

Build artifact directory for the repos shelf.

## Overview

Temporary build output directory. Contains compiled artifacts from various
projects in the shelf.

## Contents

- `debug/` - Debug build outputs
- `tarpaulin/` - Code coverage reports
- `tmp/` - Temporary build files
- `.rustc_info.json` - Rust build info
- `CACHEDIR.TAG` - Cache directory marker

## Usage

This directory is typically gitignored. Content is ephemeral and regenerated
on each build.