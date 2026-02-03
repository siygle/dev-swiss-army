use qrcode::render::unicode;
use qrcode::{EcLevel, QrCode};
use std::fmt;
use std::path::Path;

#[cfg(feature = "image-output")]
use image::{DynamicImage, GenericImageView, Rgb};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCorrectionLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl Default for ErrorCorrectionLevel {
    fn default() -> Self {
        Self::Medium
    }
}

impl ErrorCorrectionLevel {
    fn to_qrcode_level(self) -> EcLevel {
        match self {
            ErrorCorrectionLevel::Low => EcLevel::L,
            ErrorCorrectionLevel::Medium => EcLevel::M,
            ErrorCorrectionLevel::Quartile => EcLevel::Q,
            ErrorCorrectionLevel::High => EcLevel::H,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    Png,
    Svg,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Terminal
    }
}

#[derive(Debug, Clone)]
pub struct QrConfig {
    pub content: String,
    pub error_correction: ErrorCorrectionLevel,
    pub quiet_zone: bool,
    pub invert: bool,
}

impl Default for QrConfig {
    fn default() -> Self {
        Self {
            content: String::new(),
            error_correction: ErrorCorrectionLevel::Medium,
            quiet_zone: true,
            invert: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub scale: u32,
    pub dark_color: [u8; 3],
    pub light_color: [u8; 3],
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            scale: 8,
            dark_color: [0, 0, 0],       // black
            light_color: [255, 255, 255], // white
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogoConfig {
    pub path: String,
    pub size_percent: u8,
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            size_percent: 20,
        }
    }
}

#[derive(Debug)]
pub enum QrError {
    ContentTooLarge,
    EmptyContent,
    EncodingFailed(String),
    InvalidLogoPath(String),
    LogoTooLarge,
    ImageProcessingFailed(String),
    IoError(String),
    InvalidColor(String),
    BackgroundTooSmall,
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QrError::ContentTooLarge => {
                write!(f, "Content is too large for QR code encoding")
            }
            QrError::EmptyContent => {
                write!(f, "Content cannot be empty")
            }
            QrError::EncodingFailed(msg) => {
                write!(f, "QR code encoding failed: {}", msg)
            }
            QrError::InvalidLogoPath(path) => {
                write!(f, "Failed to load logo image: {}", path)
            }
            QrError::LogoTooLarge => {
                write!(f, "Logo size must be between 5% and 30% of QR code")
            }
            QrError::ImageProcessingFailed(msg) => {
                write!(f, "Image processing failed: {}", msg)
            }
            QrError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
            QrError::InvalidColor(color) => {
                write!(f, "Invalid color format: {}", color)
            }
            QrError::BackgroundTooSmall => {
                write!(f, "Background image is too small for QR code")
            }
        }
    }
}

impl std::error::Error for QrError {}

pub fn generate_qr(config: &QrConfig) -> Result<QrCode, QrError> {
    if config.content.is_empty() {
        return Err(QrError::EmptyContent);
    }

    let ec_level = config.error_correction.to_qrcode_level();

    QrCode::with_error_correction_level(&config.content, ec_level).map_err(|e| {
        if e.to_string().contains("data too long") {
            QrError::ContentTooLarge
        } else {
            QrError::EncodingFailed(e.to_string())
        }
    })
}

pub fn render_to_terminal(qr: &QrCode, config: &QrConfig) -> String {
    let mut renderer = qr.render::<unicode::Dense1x2>();

    if !config.quiet_zone {
        renderer.quiet_zone(false);
    }

    if config.invert {
        renderer
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark);
    }

    renderer.build()
}

pub fn parse_color(color: &str) -> Result<[u8; 3], QrError> {
    let color = color.trim();

    // Handle named colors
    match color.to_lowercase().as_str() {
        "black" => return Ok([0, 0, 0]),
        "white" => return Ok([255, 255, 255]),
        "red" => return Ok([255, 0, 0]),
        "green" => return Ok([0, 255, 0]),
        "blue" => return Ok([0, 0, 255]),
        _ => {}
    }

    // Handle hex colors
    let hex = color.strip_prefix('#').unwrap_or(color);

    if hex.len() != 6 {
        return Err(QrError::InvalidColor(color.to_string()));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| QrError::InvalidColor(color.to_string()))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| QrError::InvalidColor(color.to_string()))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| QrError::InvalidColor(color.to_string()))?;

    Ok([r, g, b])
}

#[cfg(feature = "image-output")]
pub fn render_to_image(qr: &QrCode, config: &ImageConfig) -> DynamicImage {
    let dark = Rgb(config.dark_color);
    let light = Rgb(config.light_color);

    let image = qr
        .render::<Rgb<u8>>()
        .quiet_zone(true)
        .module_dimensions(config.scale, config.scale)
        .dark_color(dark)
        .light_color(light)
        .build();

    DynamicImage::ImageRgb8(image)
}

#[cfg(feature = "image-output")]
pub fn overlay_logo(qr_image: &mut DynamicImage, logo_config: &LogoConfig) -> Result<(), QrError> {
    if logo_config.size_percent < 5 || logo_config.size_percent > 30 {
        return Err(QrError::LogoTooLarge);
    }

    let logo = image::open(Path::new(&logo_config.path))
        .map_err(|e| QrError::InvalidLogoPath(format!("{}: {}", logo_config.path, e)))?;

    let (qr_width, qr_height) = qr_image.dimensions();
    let max_logo_size = (qr_width as f32 * logo_config.size_percent as f32 / 100.0) as u32;

    let logo_resized = logo.resize(
        max_logo_size,
        max_logo_size,
        image::imageops::FilterType::Lanczos3,
    );

    let (logo_width, logo_height) = logo_resized.dimensions();
    let x = (qr_width - logo_width) / 2;
    let y = (qr_height - logo_height) / 2;

    image::imageops::overlay(qr_image, &logo_resized, x as i64, y as i64);

    Ok(())
}

#[cfg(feature = "image-output")]
pub fn overlay_on_background(
    qr: &QrCode,
    background_path: &str,
    image_config: &ImageConfig,
) -> Result<DynamicImage, QrError> {
    let mut background = image::open(Path::new(background_path))
        .map_err(|e| QrError::InvalidLogoPath(format!("{}: {}", background_path, e)))?;

    let (bg_width, bg_height) = background.dimensions();

    // Calculate QR code size to fit in background (with some margin)
    let qr_modules = qr.width();
    let margin = 20u32;
    let available_size = bg_width.min(bg_height).saturating_sub(margin * 2);
    let scale = available_size / qr_modules as u32;

    if scale < 2 {
        return Err(QrError::BackgroundTooSmall);
    }

    let qr_config = ImageConfig {
        scale,
        ..image_config.clone()
    };

    let qr_image = render_to_image(qr, &qr_config);
    let (qr_width, qr_height) = qr_image.dimensions();

    // Center QR on background
    let x = (bg_width - qr_width) / 2;
    let y = (bg_height - qr_height) / 2;

    image::imageops::overlay(&mut background, &qr_image, x as i64, y as i64);

    Ok(background)
}

#[cfg(feature = "image-output")]
pub fn render_to_svg(qr: &QrCode, config: &ImageConfig) -> String {
    let dark_hex = format!(
        "#{:02x}{:02x}{:02x}",
        config.dark_color[0], config.dark_color[1], config.dark_color[2]
    );
    let light_hex = format!(
        "#{:02x}{:02x}{:02x}",
        config.light_color[0], config.light_color[1], config.light_color[2]
    );

    qr.render()
        .quiet_zone(true)
        .dark_color(qrcode::render::svg::Color(&dark_hex))
        .light_color(qrcode::render::svg::Color(&light_hex))
        .build()
}

#[cfg(feature = "image-output")]
pub fn save_image(image: &DynamicImage, path: &str) -> Result<(), QrError> {
    image
        .save(path)
        .map_err(|e| QrError::IoError(format!("Failed to save image: {}", e)))
}

#[cfg(feature = "ai-generation")]
pub mod ai {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    struct StabilityRequest {
        prompt: String,
        output_format: String,
        aspect_ratio: String,
    }

    #[derive(Debug, Deserialize)]
    struct StabilityResponse {
        image: String,
    }

    pub fn generate_ai_qr(
        qr: &QrCode,
        prompt: &str,
        api_key: &str,
        image_config: &ImageConfig,
    ) -> Result<DynamicImage, QrError> {
        let client = reqwest::blocking::Client::new();

        let full_prompt = format!(
            "A QR code with artistic styling: {}. The QR code pattern should remain scannable.",
            prompt
        );

        // Generate AI background using Stability AI
        let response = client
            .post("https://api.stability.ai/v2beta/stable-image/generate/core")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Accept", "application/json")
            .form(&[
                ("prompt", full_prompt.as_str()),
                ("output_format", "png"),
                ("aspect_ratio", "1:1"),
            ])
            .send()
            .map_err(|e| QrError::ImageProcessingFailed(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(QrError::ImageProcessingFailed(format!(
                "API error {}: {}",
                status, body
            )));
        }

        let result: StabilityResponse = response
            .json()
            .map_err(|e| QrError::ImageProcessingFailed(format!("Failed to parse response: {}", e)))?;

        let image_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &result.image,
        )
        .map_err(|e| QrError::ImageProcessingFailed(format!("Failed to decode image: {}", e)))?;

        let mut background = image::load_from_memory(&image_bytes)
            .map_err(|e| QrError::ImageProcessingFailed(format!("Failed to load image: {}", e)))?;

        // Overlay QR code on the AI-generated background
        let (bg_width, bg_height) = background.dimensions();
        let qr_modules = qr.width();
        let scale = (bg_width.min(bg_height) - 40) / qr_modules as u32;

        let qr_config = ImageConfig {
            scale,
            dark_color: [0, 0, 0],
            light_color: [255, 255, 255],
        };

        // Create semi-transparent QR
        let qr_image = render_to_image(qr, &qr_config);
        let (qr_width, qr_height) = qr_image.dimensions();

        let x = (bg_width - qr_width) / 2;
        let y = (bg_height - qr_height) / 2;

        // Blend QR with background
        image::imageops::overlay(&mut background, &qr_image, x as i64, y as i64);

        Ok(background)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QrConfig::default();
        assert!(config.content.is_empty());
        assert_eq!(config.error_correction, ErrorCorrectionLevel::Medium);
        assert!(config.quiet_zone);
        assert!(!config.invert);
    }

    #[test]
    fn test_generate_simple_qr() {
        let config = QrConfig {
            content: "https://example.com".to_string(),
            ..Default::default()
        };
        let qr = generate_qr(&config).unwrap();
        assert!(qr.width() > 0);
    }

    #[test]
    fn test_empty_content_error() {
        let config = QrConfig::default();
        let result = generate_qr(&config);
        assert!(matches!(result, Err(QrError::EmptyContent)));
    }

    #[test]
    fn test_terminal_render() {
        let config = QrConfig {
            content: "test".to_string(),
            ..Default::default()
        };
        let qr = generate_qr(&config).unwrap();
        let output = render_to_terminal(&qr, &config);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_inverted_render() {
        let config = QrConfig {
            content: "test".to_string(),
            invert: true,
            ..Default::default()
        };
        let qr = generate_qr(&config).unwrap();
        let normal_config = QrConfig {
            invert: false,
            ..config.clone()
        };
        let normal = render_to_terminal(&qr, &normal_config);
        let inverted = render_to_terminal(&qr, &config);
        assert_ne!(normal, inverted);
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_color("#ff5500").unwrap();
        assert_eq!(color, [255, 85, 0]);
    }

    #[test]
    fn test_parse_named_color() {
        assert_eq!(parse_color("black").unwrap(), [0, 0, 0]);
        assert_eq!(parse_color("white").unwrap(), [255, 255, 255]);
        assert_eq!(parse_color("red").unwrap(), [255, 0, 0]);
    }

    #[test]
    fn test_invalid_color() {
        let result = parse_color("invalid");
        assert!(matches!(result, Err(QrError::InvalidColor(_))));
    }

    #[test]
    fn test_error_correction_levels() {
        for level in [
            ErrorCorrectionLevel::Low,
            ErrorCorrectionLevel::Medium,
            ErrorCorrectionLevel::Quartile,
            ErrorCorrectionLevel::High,
        ] {
            let config = QrConfig {
                content: "test".to_string(),
                error_correction: level,
                ..Default::default()
            };
            let qr = generate_qr(&config);
            assert!(qr.is_ok());
        }
    }

    #[cfg(feature = "image-output")]
    mod image_tests {
        use super::*;

        #[test]
        fn test_render_to_image() {
            let config = QrConfig {
                content: "https://example.com".to_string(),
                ..Default::default()
            };
            let qr = generate_qr(&config).unwrap();
            let image_config = ImageConfig::default();
            let image = render_to_image(&qr, &image_config);
            let (width, height) = image.dimensions();
            assert!(width > 0 && height > 0);
        }

        #[test]
        fn test_render_to_svg() {
            let config = QrConfig {
                content: "test".to_string(),
                ..Default::default()
            };
            let qr = generate_qr(&config).unwrap();
            let image_config = ImageConfig::default();
            let svg = render_to_svg(&qr, &image_config);
            assert!(svg.contains("<svg"));
            assert!(svg.contains("</svg>"));
        }

        #[test]
        fn test_logo_size_validation() {
            let logo_config = LogoConfig {
                path: "test.png".to_string(),
                size_percent: 50, // Too large
            };
            let config = QrConfig {
                content: "test".to_string(),
                ..Default::default()
            };
            let qr = generate_qr(&config).unwrap();
            let image_config = ImageConfig::default();
            let mut image = render_to_image(&qr, &image_config);
            let result = overlay_logo(&mut image, &logo_config);
            assert!(matches!(result, Err(QrError::LogoTooLarge)));
        }
    }
}
