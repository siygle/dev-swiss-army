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
