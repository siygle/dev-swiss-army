# Password Generator

Generate secure random passwords with customizable character sets and exclusions.

## Usage

```bash
dev-swiss password [OPTIONS]
```

## Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--length` | `-l` | Password length | 16 |
| `--count` | `-n` | Number of passwords to generate | 1 |
| `--no-uppercase` | | Exclude uppercase letters (A-Z) | false |
| `--no-lowercase` | | Exclude lowercase letters (a-z) | false |
| `--no-numbers` | | Exclude numbers (0-9) | false |
| `--no-symbols` | | Exclude symbols | false |
| `--no-ambiguous` | | Exclude ambiguous characters (0O1lI) | false |
| `--exclude` | | Custom characters to exclude | "" |

## Character Sets

By default, passwords include characters from all sets:

- **Uppercase**: `ABCDEFGHIJKLMNOPQRSTUVWXYZ`
- **Lowercase**: `abcdefghijklmnopqrstuvwxyz`
- **Numbers**: `0123456789`
- **Symbols**: `!@#$%^&*()_+-=[]{}|;:,.<>?`

### Ambiguous Characters

The `--no-ambiguous` flag excludes characters that look similar in many fonts:
- `0` (zero) and `O` (capital O)
- `1` (one), `l` (lowercase L), and `I` (capital I)

## Examples

### Basic Usage

```bash
# Generate a default 16-character password
dev-swiss password
# Output: Kj#9mP$xL2nQ@8vR

# Generate a longer password
dev-swiss password -l 32
```

### Multiple Passwords

```bash
# Generate 5 passwords
dev-swiss password -n 5
```

### Restricted Character Sets

```bash
# Letters and numbers only (no symbols)
dev-swiss password --no-symbols

# Numbers only (PIN-style)
dev-swiss password --no-uppercase --no-lowercase --no-symbols

# Lowercase and numbers only
dev-swiss password --no-uppercase --no-symbols
```

### Readable Passwords

```bash
# Exclude ambiguous characters for easier reading/typing
dev-swiss password --no-ambiguous

# Exclude specific characters
dev-swiss password --exclude "{}[]"
```

### Combined Options

```bash
# 20-char password, no symbols, no ambiguous chars
dev-swiss password -l 20 --no-symbols --no-ambiguous

# Generate 3 PINs of 6 digits each
dev-swiss password -l 6 -n 3 --no-uppercase --no-lowercase --no-symbols
```

## Error Handling

The command will exit with an error if:

1. **No character sets enabled**: At least one of uppercase, lowercase, numbers, or symbols must be enabled
2. **Empty character pool**: All characters have been excluded via `--exclude` or `--no-ambiguous`

```bash
# This will fail - no character sets
dev-swiss password --no-uppercase --no-lowercase --no-numbers --no-symbols
# Error: At least one character set must be enabled
```
