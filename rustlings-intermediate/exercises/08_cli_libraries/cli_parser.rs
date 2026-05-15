// Typing lecture: useful CLI libraries.
//
// Real projects often use `clap` to derive this parser from a struct. Here we
// implement the parser manually so the data model and error handling are clear
// without needing external crates.
//
// TODO: Parse the flags into the `Args` struct and report structured errors.

#[derive(Debug, PartialEq, Eq)]
struct Args {
    input: String,
    output: Option<String>,
    delimiter: char,
    header: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    MissingInput,
    MissingValue(&'static str),
    InvalidDelimiter(String),
    UnknownFlag(String),
    TooManyInputs(Vec<String>),
}

fn parse_args<I, S>(args: I) -> Result<Args, ParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    todo!("parse input, output, delimiter, and header flag");
}

fn main() {
    let args = parse_args(["--header", "--delimiter", ";", "input.csv"]);
    println!("{args:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_input_only() {
        let args = parse_args(["data.csv"]).unwrap();

        assert_eq!(
            args,
            Args {
                input: String::from("data.csv"),
                output: None,
                delimiter: ',',
                header: false,
            }
        );
    }

    #[test]
    fn parses_optional_flags() {
        let args = parse_args([
            "--header",
            "--delimiter",
            ";",
            "--output",
            "clean.csv",
            "raw.csv",
        ])
        .unwrap();

        assert_eq!(args.input, "raw.csv");
        assert_eq!(args.output.as_deref(), Some("clean.csv"));
        assert_eq!(args.delimiter, ';');
        assert!(args.header);
    }

    #[test]
    fn reports_parser_errors() {
        assert_eq!(
            parse_args(["--output"]),
            Err(ParseError::MissingValue("--output"))
        );
        assert_eq!(
            parse_args(["--delimiter", "::", "data.csv"]),
            Err(ParseError::InvalidDelimiter(String::from("::")))
        );
        assert_eq!(
            parse_args(["--unknown", "data.csv"]),
            Err(ParseError::UnknownFlag(String::from("--unknown")))
        );
        assert_eq!(
            parse_args(["one.csv", "two.csv"]),
            Err(ParseError::TooManyInputs(vec![
                String::from("one.csv"),
                String::from("two.csv")
            ]))
        );
        assert_eq!(parse_args(["--header"]), Err(ParseError::MissingInput));
    }
}
