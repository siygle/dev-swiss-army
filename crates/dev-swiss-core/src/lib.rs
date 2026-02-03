pub mod password;
pub mod qrcode;

pub use password::{generate_password, PasswordConfig, PasswordError};
pub use qrcode::{
    generate_qr, parse_color, render_to_terminal, ErrorCorrectionLevel, ImageConfig, LogoConfig,
    OutputFormat, QrConfig, QrError,
};

#[cfg(feature = "image-output")]
pub use qrcode::{overlay_logo, overlay_on_background, render_to_image, render_to_svg, save_image};

#[cfg(feature = "ai-generation")]
pub use qrcode::ai::generate_ai_qr;
