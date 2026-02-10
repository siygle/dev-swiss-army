# Convert Command

Convert files between formats.

## Usage

```bash
dev-swiss convert --from <FORMAT> --to <FORMAT> <INPUT> <OUTPUT>
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--from` | `-f` | Source format (required) |
| `--to` | `-t` | Target format (required) |
| `--force` | | Overwrite output if it exists |
| `--verbose` | `-v` | Show detailed info and warnings |

## Supported Conversions

| From | To | Notes |
|------|-----|-------|
| PDF | DOCX | Extracts text content; images and complex layouts not preserved |

## Examples

### Basic PDF to DOCX conversion

```bash
dev-swiss convert -f pdf -t docx report.pdf report.docx
```

### Overwrite existing output

```bash
dev-swiss convert -f pdf -t docx --force input.pdf output.docx
```

### Verbose mode with warnings

```bash
dev-swiss convert -f pdf -t docx --verbose input.pdf output.docx
```

## Limitations

The PDF to DOCX conversion extracts text content only:

- Images are not extracted
- Complex layouts may not preserve formatting
- Tables convert as plain text
- Scanned PDFs (image-based) will produce empty output

For high-fidelity conversion, consider using dedicated tools like LibreOffice or Adobe Acrobat.
