# Convert Command Design

## Overview

Add a `convert` command to dev-swiss CLI for file format conversion, starting with PDF to DOCX.

## Decisions

- **Approach**: Pure Rust libraries (self-contained, text-focused extraction)
- **Interface**: Explicit format flags (`--from`, `--to`)
- **Output handling**: Fail by default if output exists, `--force` to override
- **Verbosity**: Silent by default, `--verbose` shows warnings

## Command Interface

```
dev-swiss convert --from pdf --to docx input.pdf output.docx
```

**Arguments:**
- `--from <FORMAT>` / `-f` - Source format (required): `pdf`
- `--to <FORMAT>` / `-t` - Target format (required): `docx`
- `<INPUT>` - Input file path (required)
- `<OUTPUT>` - Output file path (required)

**Flags:**
- `--force` - Overwrite output file if it exists
- `--verbose` / `-v` - Show detailed conversion info and warnings

**Example usage:**
```bash
dev-swiss convert -f pdf -t docx report.pdf report.docx
dev-swiss convert -f pdf -t docx --force --verbose report.pdf report.docx
```

## Architecture

### File Structure

```
crates/dev-swiss-core/src/convert.rs    # Core conversion logic
crates/dev-swiss/src/commands/convert.rs # CLI command handler
docs/convert.md                          # Documentation
```

### Dependencies

Add to `dev-swiss-core/Cargo.toml`:
- `pdf-extract` - PDF text extraction
- `docx-rs` - DOCX file creation

### Feature Flag

`convert` (default enabled) - allows exclusion if not needed

## Core Types

```rust
// Configuration
pub struct ConvertConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub from_format: Format,
    pub to_format: Format,
    pub force: bool,
    pub verbose: bool,
}

pub enum Format {
    Pdf,
    Docx,
}

// Result with stats
pub struct ConvertResult {
    pub pages_processed: usize,
    pub warnings: Vec<String>,
}

// Errors
pub enum ConvertError {
    UnsupportedConversion { from: Format, to: Format },
    InputNotFound(PathBuf),
    OutputExists(PathBuf),
    PdfReadError(String),
    DocxWriteError(String),
    IoError(std::io::Error),
}
```

## Conversion Logic

1. Read PDF using `pdf-extract` to get text content per page
2. Create DOCX using `docx-rs` with extracted text
3. Each PDF page becomes a section in the DOCX
4. Track warnings for unsupported elements (images, tables, etc.)

## CLI Implementation

```rust
#[derive(Args)]
pub struct Args {
    #[arg(short, long, value_enum)]
    pub from: Format,

    #[arg(short, long, value_enum)]
    pub to: Format,

    pub input: PathBuf,
    pub output: PathBuf,

    #[arg(long, default_value = "false")]
    pub force: bool,

    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}
```

**Run function flow:**
1. Validate input file exists
2. Check output doesn't exist (unless `--force`)
3. Build `ConvertConfig` from args
4. Call core `convert()` function
5. Print result (pages processed, warnings if verbose)

## Limitations

Pure Rust PDF extraction has limitations:
- Images are not extracted
- Complex layouts may not preserve formatting
- Tables convert as plain text
- Primarily text-focused extraction

These limitations are communicated via warnings in verbose mode.
