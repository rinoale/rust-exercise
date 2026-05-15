// Typing lecture: systems tooling architecture.
//
// A production CLI often combines clap-style command parsing, sqlx-style
// persistence, tracing-style logs, serde-style exports, and anyhow-style error
// propagation. This exercise models the architecture with plain Rust enums.
//
// TODO: Model the project structure and command parser.

#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    Pending,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Add { title: String },
    List { status: Option<Status> },
    Complete { id: u64 },
    Delete { id: u64 },
    ExportJson,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseError {
    Empty,
    Unknown(String),
    MissingArg(&'static str),
    InvalidId(String),
    InvalidStatus(String),
}

fn project_structure() -> Vec<&'static str> {
    todo!("return the key files for a task-manager CLI project");
}

fn parse_command(tokens: &[&str]) -> Result<Command, ParseError> {
    todo!("parse add/list/complete/delete/export-json subcommands");
}

fn main() {
    println!("{:?}", project_structure());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_project_files() {
        let files = project_structure();

        assert!(files.contains(&"Cargo.toml"));
        assert!(files.contains(&"src/main.rs"));
        assert!(files.contains(&"src/cli.rs"));
        assert!(files.contains(&"src/db.rs"));
        assert!(files.contains(&"src/task.rs"));
        assert!(files.contains(&"src/logging.rs"));
    }

    #[test]
    fn parses_core_commands() {
        assert_eq!(
            parse_command(&["add", "write tests"]),
            Ok(Command::Add {
                title: String::from("write tests")
            })
        );
        assert_eq!(
            parse_command(&["list", "--status", "done"]),
            Ok(Command::List {
                status: Some(Status::Done)
            })
        );
        assert_eq!(
            parse_command(&["complete", "42"]),
            Ok(Command::Complete { id: 42 })
        );
        assert_eq!(
            parse_command(&["delete", "7"]),
            Ok(Command::Delete { id: 7 })
        );
        assert_eq!(parse_command(&["export-json"]), Ok(Command::ExportJson));
    }

    #[test]
    fn reports_parse_errors() {
        assert_eq!(parse_command(&[]), Err(ParseError::Empty));
        assert_eq!(
            parse_command(&["complete", "abc"]),
            Err(ParseError::InvalidId(String::from("abc")))
        );
        assert_eq!(
            parse_command(&["list", "--status", "blocked"]),
            Err(ParseError::InvalidStatus(String::from("blocked")))
        );
    }
}
