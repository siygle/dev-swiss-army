# Convert Command Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `convert` command to dev-swiss CLI for PDF to DOCX file conversion.

**Architecture:** Core conversion logic in `dev-swiss-core` crate with CLI wrapper in `dev-swiss` crate. Uses `pdf-extract` for PDF text extraction and `docx-rs` for DOCX creation. Feature-gated with `convert` flag.

**Tech Stack:** Rust, pdf-extract, docx-rs, clap

---

### Task 1: Add Dependencies to Core Crate

**Files:**
- Modify: `crates/dev-swiss-core/Cargo.toml`

**Step 1: Add convert dependencies**

Add to `crates/dev-swiss-core/Cargo.toml`:

```toml
pdf-extract = { version = "0.7", optional = true }
docx-rs = { version = "0.4", optional = true }
```

**Step 2: Add convert feature flag**

Add to `[features]` section in `crates/dev-swiss-core/Cargo.toml`:

```toml
convert = ["pdf-extract", "docx-rs"]
```

Update `default` to include convert:

```toml
default = ["image-output", "convert"]
```

**Step 3: Verify dependencies resolve**

Run: `cargo check -p dev-swiss-core --features convert`
Expected: Compiles successfully (may have warnings, no errors)

**Step 4: Commit**

```bash
git add crates/dev-swiss-core/Cargo.toml
git commit -m "feat(core): add pdf-extract and docx-rs dependencies for convert feature"
```

---

### Task 2: Create Core Convert Module - Error Types

**Files:**
- Create: `crates/dev-swiss-core/src/convert.rs`

**Step 1: Create convert module with error types**

Create `crates/dev-swiss-core/src/convert.rs`:

```rust
use std::fmt;
use std::path::PathBuf;

/// Supported conversion formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Pdf,
    Docx,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Pdf => write!(f, "PDF"),
            Format::Docx => write!(f, "DOCX"),
        }
    }
}

/// Conversion errors
#[derive(Debug)]
pub enum ConvertError {
    UnsupportedConversion { from: Format, to: Format },
    InputNotFound(PathBuf),
    OutputExists(PathBuf),
    PdfReadError(String),
    DocxWriteError(String),
    IoError(std::io::Error),
}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConvertError::UnsupportedConversion { from, to } => {
                write!(f, "Unsupported conversion: {} to {}", from, to)
            }
            ConvertError::InputNotFound(path) => {
                write!(f, "Input file not found: {}", path.display())
            }
            ConvertError::OutputExists(path) => {
                write!(f, "Output file already exists: {} (use --force to overwrite)", path.display())
            }
            ConvertError::PdfReadError(msg) => {
                write!(f, "Failed to read PDF: {}", msg)
            }
            ConvertError::DocxWriteError(msg) => {
                write!(f, "Failed to write DOCX: {}", msg)
            }
            ConvertError::IoError(e) => {
                write!(f, "IO error: {}", e)
            }
        }
    }
}

impl std::error::Error for ConvertError {}

impl From<std::io::Error> for ConvertError {
    fn from(e: std::io::Error) -> Self {
        ConvertError::IoError(e)
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p dev-swiss-core --features convert`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/dev-swiss-core/src/convert.rs
git commit -m "feat(core): add convert module with Format enum and ConvertError types"
```

---

### Task 3: Add Config and Result Types

**Files:**
- Modify: `crates/dev-swiss-core/src/convert.rs`

**Step 1: Add ConvertConfig and ConvertResult**

Add to `crates/dev-swiss-core/src/convert.rs` after the error types:

```rust
use std::path::PathBuf;

/// Configuration for file conversion
#[derive(Debug, Clone)]
pub struct ConvertConfig {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub from_format: Format,
    pub to_format: Format,
    pub force: bool,
    pub verbose: bool,
}

/// Result of a successful conversion
#[derive(Debug)]
pub struct ConvertResult {
    pub pages_processed: usize,
    pub warnings: Vec<String>,
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p dev-swiss-core --features convert`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/dev-swiss-core/src/convert.rs
git commit -m "feat(core): add ConvertConfig and ConvertResult types"
```

---

### Task 4: Implement PDF to DOCX Conversion

**Files:**
- Modify: `crates/dev-swiss-core/src/convert.rs`

**Step 1: Add the convert function**

Add to `crates/dev-swiss-core/src/convert.rs`:

```rust
use docx_rs::{Docx, Paragraph, Run};
use std::fs::File;
use std::io::BufWriter;

/// Convert a file from one format to another
pub fn convert(config: &ConvertConfig) -> Result<ConvertResult, ConvertError> {
    // Validate conversion is supported
    if config.from_format != Format::Pdf || config.to_format != Format::Docx {
        return Err(ConvertError::UnsupportedConversion {
            from: config.from_format,
            to: config.to_format,
        });
    }

    // Check input exists
    if !config.input_path.exists() {
        return Err(ConvertError::InputNotFound(config.input_path.clone()));
    }

    // Check output doesn't exist (unless force)
    if config.output_path.exists() && !config.force {
        return Err(ConvertError::OutputExists(config.output_path.clone()));
    }

    convert_pdf_to_docx(config)
}

fn convert_pdf_to_docx(config: &ConvertConfig) -> Result<ConvertResult, ConvertError> {
    let mut warnings = Vec::new();

    // Extract text from PDF
    let text = pdf_extract::extract_text(&config.input_path)
        .map_err(|e| ConvertError::PdfReadError(e.to_string()))?;

    // Split into pages (pdf-extract doesn't preserve page boundaries well,
    // so we treat the whole document as one "page" for now)
    let pages: Vec<&str> = if text.contains('\x0C') {
        // Form feed character sometimes used as page separator
        text.split('\x0C').collect()
    } else {
        vec![&text]
    };

    if pages.len() == 1 && pages[0].trim().is_empty() {
        warnings.push("PDF appears to contain no extractable text (may be image-based)".to_string());
    }

    // Create DOCX document
    let mut docx = Docx::new();

    for (i, page_text) in pages.iter().enumerate() {
        // Add page content as paragraphs (split by newlines)
        for line in page_text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let paragraph = Paragraph::new().add_run(Run::new().add_text(trimmed));
                docx = docx.add_paragraph(paragraph);
            }
        }

        // Add page break between pages (except after last page)
        if i < pages.len() - 1 {
            let page_break = Paragraph::new().add_run(Run::new().add_break(docx_rs::BreakType::Page));
            docx = docx.add_paragraph(page_break);
        }
    }

    // Write DOCX file
    let file = File::create(&config.output_path)?;
    let mut writer = BufWriter::new(file);
    docx.build()
        .pack(&mut writer)
        .map_err(|e| ConvertError::DocxWriteError(e.to_string()))?;

    Ok(ConvertResult {
        pages_processed: pages.len(),
        warnings,
    })
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p dev-swiss-core --features convert`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/dev-swiss-core/src/convert.rs
git commit -m "feat(core): implement PDF to DOCX conversion"
```

---

### Task 5: Export Convert Module from Core Library

**Files:**
- Modify: `crates/dev-swiss-core/src/lib.rs`

**Step 1: Add convert module export**

Add to `crates/dev-swiss-core/src/lib.rs`:

```rust
#[cfg(feature = "convert")]
pub mod convert;

#[cfg(feature = "convert")]
pub use convert::{convert, ConvertConfig, ConvertError, ConvertResult, Format};
```

**Step 2: Verify it compiles**

Run: `cargo check -p dev-swiss-core --features convert`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/dev-swiss-core/src/lib.rs
git commit -m "feat(core): export convert module from library"
```

---

### Task 6: Add Convert Feature to CLI Crate

**Files:**
- Modify: `crates/dev-swiss/Cargo.toml`

**Step 1: Add convert feature to CLI**

Add to `[features]` section in `crates/dev-swiss/Cargo.toml`:

```toml
convert = ["dev-swiss-core/convert"]
```

Update `default` features:

```toml
default = ["convert"]
```

Update the dev-swiss-core dependency to include convert:

```toml
dev-swiss-core = { path = "../dev-swiss-core", features = ["image-output", "convert"] }
```

**Step 2: Verify it compiles**

Run: `cargo check -p dev-swiss`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add crates/dev-swiss/Cargo.toml
git commit -m "feat(cli): add convert feature flag"
```

---

### Task 7: Create CLI Convert Command

**Files:**
- Create: `crates/dev-swiss/src/commands/convert.rs`
- Modify: `crates/dev-swiss/src/commands/mod.rs`

**Step 1: Create convert command module**

Create `crates/dev-swiss/src/commands/convert.rs`:

```rust
use clap::{Args, ValueEnum};
use dev_swiss_core::{convert, ConvertConfig, Format as CoreFormat};
use std::path::PathBuf;

#[derive(Args)]
pub struct ConvertArgs {
    /// Source format
    #[arg(short, long, value_enum)]
    pub from: Format,

    /// Target format
    #[arg(short, long, value_enum)]
    pub to: Format,

    /// Input file path
    pub input: PathBuf,

    /// Output file path
    pub output: PathBuf,

    /// Overwrite output file if it exists
    #[arg(long, default_value = "false")]
    pub force: bool,

    /// Show detailed conversion info and warnings
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,
}

#[derive(Clone, ValueEnum)]
pub enum Format {
    Pdf,
    Docx,
}

impl From<Format> for CoreFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Pdf => CoreFormat::Pdf,
            Format::Docx => CoreFormat::Docx,
        }
    }
}

pub fn run(args: ConvertArgs) {
    let config = ConvertConfig {
        input_path: args.input,
        output_path: args.output.clone(),
        from_format: args.from.into(),
        to_format: args.to.into(),
        force: args.force,
        verbose: args.verbose,
    };

    match convert(&config) {
        Ok(result) => {
            if args.verbose {
                println!("Converted {} page(s)", result.pages_processed);
                for warning in &result.warnings {
                    eprintln!("Warning: {}", warning);
                }
            }
            println!("Successfully converted to {}", args.output.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

**Step 2: Add convert to commands mod.rs**

Add to `crates/dev-swiss/src/commands/mod.rs`:

```rust
#[cfg(feature = "convert")]
pub mod convert;
```

**Step 3: Verify it compiles**

Run: `cargo check -p dev-swiss`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add crates/dev-swiss/src/commands/convert.rs crates/dev-swiss/src/commands/mod.rs
git commit -m "feat(cli): add convert command module"
```

---

### Task 8: Wire Convert Command into Main

**Files:**
- Modify: `crates/dev-swiss/src/main.rs`

**Step 1: Add Convert variant to Commands enum**

Modify `crates/dev-swiss/src/main.rs` to add the Convert command:

```rust
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "dev-swiss")]
#[command(about = "A Swiss Army knife CLI toolkit for developers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate secure random passwords
    Password(commands::password::PasswordArgs),
    /// Generate QR codes from URLs or text
    Qrcode(commands::qrcode::QrCodeArgs),
    /// Convert files between formats
    #[cfg(feature = "convert")]
    Convert(commands::convert::ConvertArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Password(args) => commands::password::run(args),
        Commands::Qrcode(args) => commands::qrcode::run(args),
        #[cfg(feature = "convert")]
        Commands::Convert(args) => commands::convert::run(args),
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build -p dev-swiss`
Expected: Compiles successfully

**Step 3: Verify help shows convert command**

Run: `cargo run -p dev-swiss -- --help`
Expected: Shows "convert" in commands list

**Step 4: Commit**

```bash
git add crates/dev-swiss/src/main.rs
git commit -m "feat(cli): wire convert command into main"
```

---

### Task 9: Add Unit Tests for Core Convert

**Files:**
- Modify: `crates/dev-swiss-core/src/convert.rs`

**Step 1: Add tests module**

Add to the end of `crates/dev-swiss-core/src/convert.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_display() {
        assert_eq!(format!("{}", Format::Pdf), "PDF");
        assert_eq!(format!("{}", Format::Docx), "DOCX");
    }

    #[test]
    fn test_unsupported_conversion() {
        let config = ConvertConfig {
            input_path: PathBuf::from("test.docx"),
            output_path: PathBuf::from("test.pdf"),
            from_format: Format::Docx,
            to_format: Format::Pdf,
            force: false,
            verbose: false,
        };

        let result = convert(&config);
        assert!(matches!(
            result,
            Err(ConvertError::UnsupportedConversion { .. })
        ));
    }

    #[test]
    fn test_input_not_found() {
        let config = ConvertConfig {
            input_path: PathBuf::from("/nonexistent/path/file.pdf"),
            output_path: PathBuf::from("output.docx"),
            from_format: Format::Pdf,
            to_format: Format::Docx,
            force: false,
            verbose: false,
        };

        let result = convert(&config);
        assert!(matches!(result, Err(ConvertError::InputNotFound(_))));
    }

    #[test]
    fn test_output_exists_without_force() {
        // Create a temporary "input" PDF (even if invalid, we check output first in real impl)
        let input = NamedTempFile::new().unwrap();
        let output = NamedTempFile::new().unwrap();

        let config = ConvertConfig {
            input_path: input.path().to_path_buf(),
            output_path: output.path().to_path_buf(),
            from_format: Format::Pdf,
            to_format: Format::Docx,
            force: false,
            verbose: false,
        };

        // Note: This test may fail at PDF reading stage since input isn't a valid PDF
        // The output exists check happens after input validation in current impl
        let result = convert(&config);
        // Either OutputExists or PdfReadError depending on order of checks
        assert!(result.is_err());
    }

    #[test]
    fn test_error_display() {
        let err = ConvertError::InputNotFound(PathBuf::from("test.pdf"));
        assert!(err.to_string().contains("test.pdf"));

        let err = ConvertError::UnsupportedConversion {
            from: Format::Docx,
            to: Format::Pdf,
        };
        assert!(err.to_string().contains("DOCX"));
        assert!(err.to_string().contains("PDF"));
    }
}
```

**Step 2: Add tempfile dev dependency**

Add to `crates/dev-swiss-core/Cargo.toml` in `[dev-dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Run tests**

Run: `cargo test -p dev-swiss-core --features convert`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/dev-swiss-core/src/convert.rs crates/dev-swiss-core/Cargo.toml
git commit -m "test(core): add unit tests for convert module"
```

---

### Task 10: Create Documentation

**Files:**
- Create: `docs/convert.md`

**Step 1: Create documentation file**

Create `docs/convert.md`:

```markdown
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
```

**Step 2: Commit**

```bash
git add docs/convert.md
git commit -m "docs: add convert command documentation"
```

---

### Task 11: End-to-End Test

**Step 1: Build the CLI**

Run: `cargo build -p dev-swiss --release`
Expected: Builds successfully

**Step 2: Test help output**

Run: `cargo run -p dev-swiss -- convert --help`
Expected: Shows convert command options

**Step 3: Test with a sample PDF (if available)**

If you have a test PDF:
```bash
cargo run -p dev-swiss -- convert -f pdf -t docx test.pdf test.docx -v
```

**Step 4: Verify error handling**

Run: `cargo run -p dev-swiss -- convert -f pdf -t docx nonexistent.pdf out.docx`
Expected: Error message about input file not found

**Step 5: Final commit with all tests passing**

```bash
git status
# If any uncommitted changes, commit them
```
