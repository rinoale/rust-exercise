// Typing lecture: useful CLI libraries.

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
    let mut output = None;
    let mut delimiter = ',';
    let mut header = false;
    let mut inputs = Vec::new();

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg = arg.as_ref();

        match arg {
            "--header" => header = true,
            "--output" => {
                let value = args
                    .next()
                    .ok_or(ParseError::MissingValue("--output"))?
                    .as_ref()
                    .to_string();
                output = Some(value);
            }
            "--delimiter" => {
                let value = args
                    .next()
                    .ok_or(ParseError::MissingValue("--delimiter"))?
                    .as_ref()
                    .to_string();
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(ch), None) => delimiter = ch,
                    _ => return Err(ParseError::InvalidDelimiter(value)),
                }
            }
            flag if flag.starts_with('-') => {
                return Err(ParseError::UnknownFlag(flag.to_string()));
            }
            input => inputs.push(input.to_string()),
        }
    }

    match inputs.len() {
        0 => Err(ParseError::MissingInput),
        1 => Ok(Args {
            input: inputs.remove(0),
            output,
            delimiter,
            header,
        }),
        _ => Err(ParseError::TooManyInputs(inputs)),
    }
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
