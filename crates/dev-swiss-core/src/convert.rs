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
