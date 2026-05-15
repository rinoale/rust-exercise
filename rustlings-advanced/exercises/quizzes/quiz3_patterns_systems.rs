// Quiz 3: patterns and systems tools.
//
// TODO: Match problems to Rust patterns and ecosystem tools.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pattern {
    Builder,
    Typestate,
    Decorator,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Clap,
    Sqlx,
    Tracing,
    Serde,
    Rayon,
}

fn choose_pattern(problem: &str) -> Pattern {
    todo!("choose builder, typestate, decorator, or command");
}

fn choose_tool(need: &str) -> Tool {
    todo!("choose clap, sqlx, tracing, serde, or rayon");
}

fn main() {
    println!("{:?}", choose_pattern("undo redo"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_patterns() {
        assert_eq!(
            choose_pattern("step by step construction"),
            Pattern::Builder
        );
        assert_eq!(
            choose_pattern("compile time state transitions"),
            Pattern::Typestate
        );
        assert_eq!(
            choose_pattern("wrap a handler with middleware"),
            Pattern::Decorator
        );
        assert_eq!(choose_pattern("undo redo"), Pattern::Command);
    }

    #[test]
    fn chooses_tools() {
        assert_eq!(choose_tool("parse subcommands"), Tool::Clap);
        assert_eq!(choose_tool("sqlite queries"), Tool::Sqlx);
        assert_eq!(choose_tool("structured logs"), Tool::Tracing);
        assert_eq!(choose_tool("json export"), Tool::Serde);
        assert_eq!(choose_tool("data parallelism"), Tool::Rayon);
    }
}
