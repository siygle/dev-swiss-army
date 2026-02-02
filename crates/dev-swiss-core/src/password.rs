use rand::Rng;
use std::fmt;

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
const AMBIGUOUS: &str = "0O1lI";

#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
    pub exclude_chars: String,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 16,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            exclude_ambiguous: false,
            exclude_chars: String::new(),
        }
    }
}

#[derive(Debug)]
pub enum PasswordError {
    NoCharacterSets,
    EmptyCharacterPool,
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordError::NoCharacterSets => {
                write!(f, "At least one character set must be enabled")
            }
            PasswordError::EmptyCharacterPool => {
                write!(f, "No characters available after applying exclusions")
            }
        }
    }
}

impl std::error::Error for PasswordError {}

pub fn generate_password(config: &PasswordConfig) -> Result<String, PasswordError> {
    let mut charset = String::new();

    if config.uppercase {
        charset.push_str(UPPERCASE);
    }
    if config.lowercase {
        charset.push_str(LOWERCASE);
    }
    if config.numbers {
        charset.push_str(NUMBERS);
    }
    if config.symbols {
        charset.push_str(SYMBOLS);
    }

    if charset.is_empty() {
        return Err(PasswordError::NoCharacterSets);
    }

    // Remove ambiguous characters if requested
    if config.exclude_ambiguous {
        charset = charset
            .chars()
            .filter(|c| !AMBIGUOUS.contains(*c))
            .collect();
    }

    // Remove custom excluded characters
    if !config.exclude_chars.is_empty() {
        charset = charset
            .chars()
            .filter(|c| !config.exclude_chars.contains(*c))
            .collect();
    }

    if charset.is_empty() {
        return Err(PasswordError::EmptyCharacterPool);
    }

    let charset: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();

    let password: String = (0..config.length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx]
        })
        .collect();

    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PasswordConfig::default();
        assert_eq!(config.length, 16);
        assert!(config.uppercase);
        assert!(config.lowercase);
        assert!(config.numbers);
        assert!(config.symbols);
        assert!(!config.exclude_ambiguous);
        assert!(config.exclude_chars.is_empty());
    }

    #[test]
    fn test_generate_default_password() {
        let config = PasswordConfig::default();
        let password = generate_password(&config).unwrap();
        assert_eq!(password.len(), 16);
    }

    #[test]
    fn test_generate_custom_length() {
        let config = PasswordConfig {
            length: 32,
            ..Default::default()
        };
        let password = generate_password(&config).unwrap();
        assert_eq!(password.len(), 32);
    }

    #[test]
    fn test_no_symbols() {
        let config = PasswordConfig {
            symbols: false,
            ..Default::default()
        };
        let password = generate_password(&config).unwrap();
        assert!(!password.chars().any(|c| SYMBOLS.contains(c)));
    }

    #[test]
    fn test_exclude_ambiguous() {
        let config = PasswordConfig {
            exclude_ambiguous: true,
            length: 100, // Generate longer password to increase chance of ambiguous chars
            ..Default::default()
        };
        let password = generate_password(&config).unwrap();
        assert!(!password.chars().any(|c| AMBIGUOUS.contains(c)));
    }

    #[test]
    fn test_exclude_custom_chars() {
        let config = PasswordConfig {
            exclude_chars: "abc".to_string(),
            length: 100,
            ..Default::default()
        };
        let password = generate_password(&config).unwrap();
        assert!(!password.contains('a'));
        assert!(!password.contains('b'));
        assert!(!password.contains('c'));
    }

    #[test]
    fn test_no_character_sets_error() {
        let config = PasswordConfig {
            uppercase: false,
            lowercase: false,
            numbers: false,
            symbols: false,
            ..Default::default()
        };
        let result = generate_password(&config);
        assert!(matches!(result, Err(PasswordError::NoCharacterSets)));
    }

    #[test]
    fn test_only_numbers() {
        let config = PasswordConfig {
            uppercase: false,
            lowercase: false,
            numbers: true,
            symbols: false,
            ..Default::default()
        };
        let password = generate_password(&config).unwrap();
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }
}
