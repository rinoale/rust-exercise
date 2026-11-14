// Typing lecture: `FromStr` and small CLI-style config parsing.
//
// Many Rust libraries parse text into strongly typed values. This exercise
// practices `FromStr`, `Result`, and converting stringly input into domain
// types.
//
// TODO: Parse log levels and command-line-like flags.

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        todo!("parse error, warn, info, and debug");
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AppConfig {
    input: String,
    retries: u8,
    log_level: LogLevel,
}

fn parse_config(args: &[&str]) -> Result<AppConfig, String> {
    todo!("parse --input, --retries, and --log flags");
}

fn main() {
    println!("{:?}", parse_config(&["--input", "data.txt"]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_log_levels() {
        assert_eq!("error".parse::<LogLevel>(), Ok(LogLevel::Error));
        assert_eq!("WARN".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert!("trace".parse::<LogLevel>().is_err());
    }

    #[test]
    fn parses_config_with_defaults() {
        assert_eq!(
            parse_config(&["--input", "data.txt"]).unwrap(),
            AppConfig {
                input: String::from("data.txt"),
                retries: 3,
                log_level: LogLevel::Info,
            }
        );
    }

    #[test]
    fn parses_all_flags() {
        assert_eq!(
            parse_config(&["--input", "data.txt", "--retries", "5", "--log", "debug"]).unwrap(),
            AppConfig {
                input: String::from("data.txt"),
                retries: 5,
                log_level: LogLevel::Debug,
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_config(&["--retries", "2"]).is_err());
        assert!(parse_config(&["--input"]).is_err());
        assert!(parse_config(&["--input", "data.txt", "--unknown"]).is_err());
    }
}
