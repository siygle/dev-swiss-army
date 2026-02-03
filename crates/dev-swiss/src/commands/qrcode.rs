use clap::{Args, ValueEnum};
use dev_swiss_core::{
    generate_qr, parse_color, render_to_terminal, ErrorCorrectionLevel, ImageConfig, LogoConfig,
    QrConfig,
};

#[cfg(feature = "ai-generation")]
use dev_swiss_core::generate_ai_qr;

use dev_swiss_core::{overlay_logo, overlay_on_background, render_to_image, render_to_svg, save_image};

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Terminal,
    Png,
    Svg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EcLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl From<EcLevel> for ErrorCorrectionLevel {
    fn from(level: EcLevel) -> Self {
        match level {
            EcLevel::Low => ErrorCorrectionLevel::Low,
            EcLevel::Medium => ErrorCorrectionLevel::Medium,
            EcLevel::Quartile => ErrorCorrectionLevel::Quartile,
            EcLevel::High => ErrorCorrectionLevel::High,
        }
    }
}

#[derive(Args)]
pub struct QrCodeArgs {
    /// URL or text content to encode in the QR code
    content: String,

    /// Output format
    #[arg(short, long, value_enum, default_value = "terminal")]
    format: Format,

    /// Output file path (required for png/svg formats)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Error correction level
    #[arg(short = 'e', long, value_enum, default_value = "medium")]
    error_correction: EcLevel,

    /// Scale factor for image output (pixels per module)
    #[arg(short, long, default_value_t = 8)]
    scale: u32,

    /// Invert colors (dark <-> light)
    #[arg(long)]
    invert: bool,

    /// Hide quiet zone (border around QR code)
    #[arg(long)]
    no_quiet_zone: bool,

    /// Path to logo image to embed in center
    #[arg(long)]
    logo: Option<PathBuf>,

    /// Logo size as percentage of QR code (5-30)
    #[arg(long, default_value_t = 20)]
    logo_size: u8,

    /// Path to background image
    #[arg(long)]
    background: Option<PathBuf>,

    /// Dark module color (hex: #000000 or name: black)
    #[arg(long, default_value = "black")]
    dark_color: String,

    /// Light module color (hex: #FFFFFF or name: white)
    #[arg(long, default_value = "white")]
    light_color: String,

    /// AI prompt for artistic QR generation (requires API key)
    #[arg(long)]
    ai_prompt: Option<String>,

    /// Stability AI API key (or set STABILITY_API_KEY env var)
    #[arg(long, env = "STABILITY_API_KEY")]
    api_key: Option<String>,
}

pub fn run(args: QrCodeArgs) {
    // Auto-upgrade error correction when using logo
    let error_correction = if args.logo.is_some() && matches!(args.error_correction, EcLevel::Low | EcLevel::Medium) {
        eprintln!("Note: Using high error correction for logo overlay");
        ErrorCorrectionLevel::High
    } else {
        args.error_correction.into()
    };

    let qr_config = QrConfig {
        content: args.content.clone(),
        error_correction,
        quiet_zone: !args.no_quiet_zone,
        invert: args.invert,
    };

    let qr = match generate_qr(&qr_config) {
        Ok(qr) => qr,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    match args.format {
        Format::Terminal => {
            let output = render_to_terminal(&qr, &qr_config);
            println!("{}", output);
        }
        Format::Png | Format::Svg => {
            let output_path = match &args.output {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    let ext = match args.format {
                        Format::Png => "png",
                        Format::Svg => "svg",
                        _ => unreachable!(),
                    };
                    eprintln!("Error: Output path required for {} format. Use -o <path>", ext);
                    std::process::exit(1);
                }
            };

            let dark_color = match parse_color(&args.dark_color) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            let light_color = match parse_color(&args.light_color) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            let image_config = ImageConfig {
                scale: args.scale,
                dark_color,
                light_color,
            };

            match args.format {
                Format::Svg => {
                    let svg = render_to_svg(&qr, &image_config);
                    if let Err(e) = std::fs::write(&output_path, svg) {
                        eprintln!("Error: Failed to write file: {}", e);
                        std::process::exit(1);
                    }
                    println!("Saved SVG to {}", output_path);
                }
                Format::Png => {
                    #[cfg(feature = "ai-generation")]
                    if let Some(prompt) = &args.ai_prompt {
                        let api_key = match &args.api_key {
                            Some(key) => key.clone(),
                            None => {
                                eprintln!("Error: API key required for AI generation. Use --api-key or set STABILITY_API_KEY");
                                std::process::exit(1);
                            }
                        };

                        match generate_ai_qr(&qr, prompt, &api_key, &image_config) {
                            Ok(image) => {
                                if let Err(e) = save_image(&image, &output_path) {
                                    eprintln!("Error: {}", e);
                                    std::process::exit(1);
                                }
                                println!("Saved AI-styled QR to {}", output_path);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                        return;
                    }

                    #[cfg(not(feature = "ai-generation"))]
                    if args.ai_prompt.is_some() {
                        eprintln!("Error: AI generation not available. Rebuild with 'ai-generation' feature.");
                        std::process::exit(1);
                    }

                    // Handle background image
                    if let Some(bg_path) = &args.background {
                        match overlay_on_background(&qr, &bg_path.to_string_lossy(), &image_config) {
                            Ok(image) => {
                                if let Err(e) = save_image(&image, &output_path) {
                                    eprintln!("Error: {}", e);
                                    std::process::exit(1);
                                }
                                println!("Saved QR with background to {}", output_path);
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                        return;
                    }

                    // Standard image rendering
                    let mut image = render_to_image(&qr, &image_config);

                    // Handle logo overlay
                    if let Some(logo_path) = &args.logo {
                        let logo_config = LogoConfig {
                            path: logo_path.to_string_lossy().to_string(),
                            size_percent: args.logo_size,
                        };

                        if let Err(e) = overlay_logo(&mut image, &logo_config) {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }

                    if let Err(e) = save_image(&image, &output_path) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                    println!("Saved PNG to {}", output_path);
                }
                _ => unreachable!(),
            }
        }
    }
}
