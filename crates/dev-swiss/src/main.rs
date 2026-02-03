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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Password(args) => commands::password::run(args),
        Commands::Qrcode(args) => commands::qrcode::run(args),
    }
}
