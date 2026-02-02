use clap::Args;
use dev_swiss_core::{generate_password, PasswordConfig};

#[derive(Args)]
pub struct PasswordArgs {
    /// Password length
    #[arg(short, long, default_value_t = 16)]
    length: usize,

    /// Number of passwords to generate
    #[arg(short = 'n', long, default_value_t = 1)]
    count: usize,

    /// Exclude uppercase letters
    #[arg(long)]
    no_uppercase: bool,

    /// Exclude lowercase letters
    #[arg(long)]
    no_lowercase: bool,

    /// Exclude numbers
    #[arg(long)]
    no_numbers: bool,

    /// Exclude symbols
    #[arg(long)]
    no_symbols: bool,

    /// Exclude ambiguous characters (0O1lI)
    #[arg(long)]
    no_ambiguous: bool,

    /// Custom characters to exclude
    #[arg(long, default_value = "")]
    exclude: String,
}

pub fn run(args: PasswordArgs) {
    let config = PasswordConfig {
        length: args.length,
        uppercase: !args.no_uppercase,
        lowercase: !args.no_lowercase,
        numbers: !args.no_numbers,
        symbols: !args.no_symbols,
        exclude_ambiguous: args.no_ambiguous,
        exclude_chars: args.exclude,
    };

    for _ in 0..args.count {
        match generate_password(&config) {
            Ok(password) => println!("{}", password),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
