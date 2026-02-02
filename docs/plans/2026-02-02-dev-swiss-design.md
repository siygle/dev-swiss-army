# Dev-Swiss CLI Toolkit Design

**Date**: 2026-02-02
**Status**: Implemented

## Overview

A Rust CLI toolkit for developer productivity, using a subcommand-based architecture (`dev-swiss <tool>`).

## Architecture

**Workspace structure with two crates:**
- `dev-swiss-core`: Shared library with tool logic
- `dev-swiss`: CLI binary with clap-based argument parsing

This separation enables testable, reusable logic while keeping the CLI thin.

## First Tool: Password Generator

**Design decisions:**
- All character types ON by default (most secure baseline)
- Use `--no-X` flags to disable features (cleaner than boolean flags)
- Default length of 16 characters
- Ambiguous character exclusion (`0O1lI`) for readability
- Custom character exclusion support
- Multiple password generation with `-n`

**Character sets:**
- Uppercase: A-Z
- Lowercase: a-z
- Numbers: 0-9
- Symbols: `!@#$%^&*()_+-=[]{}|;:,.<>?`

## Dependencies

Kept minimal:
- `rand` for cryptographically secure random generation
- `clap` for CLI parsing

## Future Extensions

Adding new tools:
1. Add module in `dev-swiss-core/src/<tool>.rs`
2. Add command in `dev-swiss/src/commands/<tool>.rs`
3. Register subcommand in `main.rs`
4. Add documentation in `docs/<tool>.md`
