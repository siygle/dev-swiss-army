# dev-swiss

A Swiss Army knife CLI toolkit for developers.

## Installation

```bash
cargo install --path crates/dev-swiss
```

Or build from source:

```bash
cargo build --release
```

The binary will be at `target/release/dev-swiss`.

## Tools

### Password Generator

Generate secure random passwords with customizable options.

```bash
# Generate a 16-character password (default)
dev-swiss password

# Generate a 24-character password
dev-swiss password -l 24

# Generate 5 passwords
dev-swiss password -n 5

# Exclude symbols
dev-swiss password --no-symbols

# Exclude ambiguous characters (0O1lI)
dev-swiss password --no-ambiguous

# Combine options
dev-swiss password -l 20 --no-ambiguous --no-symbols -n 3
```

See [docs/password.md](docs/password.md) for detailed documentation.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- password --help
```

## License

See [LICENSE](LICENSE) for details.
