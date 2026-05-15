// Typing lecture: systems tooling architecture.

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
    vec![
        "Cargo.toml",
        "src/main.rs",
        "src/cli.rs",
        "src/db.rs",
        "src/task.rs",
        "src/logging.rs",
    ]
}

fn parse_id(raw: Option<&&str>, name: &'static str) -> Result<u64, ParseError> {
    let raw = raw.ok_or(ParseError::MissingArg(name))?;
    raw.parse()
        .map_err(|_| ParseError::InvalidId((*raw).to_string()))
}

fn parse_command(tokens: &[&str]) -> Result<Command, ParseError> {
    let Some((command, rest)) = tokens.split_first() else {
        return Err(ParseError::Empty);
    };

    match *command {
        "add" => {
            let title = rest.first().ok_or(ParseError::MissingArg("title"))?;
            Ok(Command::Add {
                title: (*title).to_string(),
            })
        }
        "list" => match rest {
            [] => Ok(Command::List { status: None }),
            ["--status", "done"] => Ok(Command::List {
                status: Some(Status::Done),
            }),
            ["--status", "pending"] => Ok(Command::List {
                status: Some(Status::Pending),
            }),
            ["--status", other] => Err(ParseError::InvalidStatus((*other).to_string())),
            [other, ..] => Err(ParseError::Unknown((*other).to_string())),
        },
        "complete" => Ok(Command::Complete {
            id: parse_id(rest.first(), "id")?,
        }),
        "delete" => Ok(Command::Delete {
            id: parse_id(rest.first(), "id")?,
        }),
        "export-json" => Ok(Command::ExportJson),
        other => Err(ParseError::Unknown(other.to_string())),
    }
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
