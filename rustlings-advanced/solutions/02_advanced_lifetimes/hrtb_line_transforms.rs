// Typing lecture: higher-ranked trait bounds.

fn trim_line(line: &str) -> &str {
    line.trim()
}

fn before_colon(line: &str) -> &str {
    match line.split_once(':') {
        Some((left, _)) => left,
        None => line,
    }
}

fn transform_lines<F>(input: &str, transform: F) -> Vec<&str>
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    input.lines().map(transform).collect()
}

fn main() {
    println!("{:?}", transform_lines(" a \n b ", trim_line));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_each_borrowed_line() {
        let input = String::from(" first \n second \n third ");

        assert_eq!(
            transform_lines(&input, trim_line),
            vec!["first", "second", "third"]
        );
    }

    #[test]
    fn extracts_prefixes_with_any_lifetime() {
        let input = String::from("name:rust\nlevel:advanced\nplain");

        assert_eq!(
            transform_lines(&input, before_colon),
            vec!["name", "level", "plain"]
        );
    }
}
