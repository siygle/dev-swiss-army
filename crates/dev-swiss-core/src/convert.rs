use docx_rs::{Docx, Paragraph, Run};
use std::fmt;
use std::fs::File;
use std::io::BufWriter;
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
