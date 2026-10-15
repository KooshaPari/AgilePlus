# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.5](https://github.com/KooshaPari/AgilePlus/compare/agileplus-transport-tests-v0.2.4...agileplus-transport-tests-v0.2.5) - 2026-10-15

### Added

- *(tests)* add full E2E workflow tests — init through ship
- *(tests)* add desktop IPC integration tests — DB layer verification
- *(tests)* add CLI transport tests — binary-level verification
- *(tests)* add transport-layer tests + fix CI coverage threshold

### Fixed

- *(machete)* remove 33 genuinely unused deps; ignore the 3 prost codegen false positives
- resolve CI release build failures

### Other

- *(fmt)* format the workspace with CI's exact toolchain
