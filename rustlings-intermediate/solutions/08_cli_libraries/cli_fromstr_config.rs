// Typing lecture: `FromStr` and small CLI-style config parsing.

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
        match value.to_ascii_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            other => Err(format!("unknown log level: {other}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AppConfig {
    input: String,
    retries: u8,
    log_level: LogLevel,
}

fn parse_config(args: &[&str]) -> Result<AppConfig, String> {
    let mut input = None;
    let mut retries = 3;
    let mut log_level = LogLevel::Info;
    let mut index = 0;

    while index < args.len() {
        match args[index] {
            "--input" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| String::from("--input requires a value"))?;
                input = Some((*value).to_string());
            }
            "--retries" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| String::from("--retries requires a value"))?;
                retries = value
                    .parse::<u8>()
                    .map_err(|error| format!("invalid retries: {error}"))?;
            }
            "--log" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| String::from("--log requires a value"))?;
                log_level = value.parse()?;
            }
            other => return Err(format!("unknown flag: {other}")),
        }

        index += 1;
    }

    let input = input.ok_or_else(|| String::from("--input is required"))?;

    Ok(AppConfig {
        input,
        retries,
        log_level,
    })
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
