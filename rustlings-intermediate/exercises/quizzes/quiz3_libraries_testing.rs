// Quiz 3: libraries, async shape, and testing.
//
// TODO: Match ecosystem jobs to tools and testing concepts.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Serde,
    Clap,
    Regex,
    Tokio,
    Reqwest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestKind {
    Unit,
    Integration,
    Doc,
}

fn choose_tool(job: &str) -> Tool {
    todo!("choose serde, clap, regex, tokio, or reqwest");
}

fn async_keyword_to_wait() -> &'static str {
    todo!("which keyword waits for a future?");
}

fn choose_test_kind(location: &str) -> TestKind {
    todo!("choose unit, integration, or doc tests");
}

fn divide_by_zero_result() -> &'static str {
    todo!("what should a fallible divide function return for zero divisors?");
}

fn main() {
    println!("{:?}", choose_tool("json"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_common_crates() {
        assert_eq!(choose_tool("json"), Tool::Serde);
        assert_eq!(choose_tool("cli arguments"), Tool::Clap);
        assert_eq!(choose_tool("pattern matching text"), Tool::Regex);
        assert_eq!(choose_tool("async runtime"), Tool::Tokio);
        assert_eq!(choose_tool("http client"), Tool::Reqwest);
    }

    #[test]
    fn knows_async_wait_keyword() {
        assert_eq!(async_keyword_to_wait(), "await");
    }

    #[test]
    fn chooses_test_kinds() {
        assert_eq!(
            choose_test_kind("same file cfg test module"),
            TestKind::Unit
        );
        assert_eq!(choose_test_kind("tests directory"), TestKind::Integration);
        assert_eq!(choose_test_kind("doc comment example"), TestKind::Doc);
    }

    #[test]
    fn knows_result_for_error_cases() {
        assert_eq!(divide_by_zero_result(), "Err");
    }
}
