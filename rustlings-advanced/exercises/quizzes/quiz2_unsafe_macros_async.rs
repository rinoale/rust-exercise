// Quiz 2: unsafe, macros, and async.
//
// TODO: Fill in the concept answers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MacroKind {
    Declarative,
    CustomDerive,
    AttributeLike,
    FunctionLike,
}

fn unsafe_superpowers() -> Vec<&'static str> {
    todo!("return the five operations unlocked by unsafe Rust");
}

fn macro_kind(name: &str) -> MacroKind {
    todo!("classify macro_rules, derive, attribute, and function-like macros");
}

fn async_fn_returns_lazy_future() -> bool {
    todo!("does an async fn body run immediately when called?");
}

fn main() {
    println!("{:?}", unsafe_superpowers());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_unsafe_superpowers() {
        assert_eq!(
            unsafe_superpowers(),
            vec![
                "dereference raw pointer",
                "call unsafe function",
                "access mutable static",
                "implement unsafe trait",
                "access union field",
            ]
        );
    }

    #[test]
    fn classifies_macro_kinds() {
        assert_eq!(macro_kind("macro_rules"), MacroKind::Declarative);
        assert_eq!(macro_kind("derive"), MacroKind::CustomDerive);
        assert_eq!(macro_kind("attribute"), MacroKind::AttributeLike);
        assert_eq!(macro_kind("function"), MacroKind::FunctionLike);
    }

    #[test]
    fn async_functions_are_lazy_until_polled() {
        assert!(async_fn_returns_lazy_future());
    }
}
