// Quiz 1: generics, traits, and lifetimes.
//
// TODO: Answer the concept questions by returning the expected values.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dispatch {
    Static,
    Dynamic,
}

fn generic_parameter_name() -> char {
    todo!("what generic type parameter name is conventional for one type?");
}

fn display_trait_bound() -> &'static str {
    todo!("which standard trait supports user-facing formatting?");
}

fn borrowed_return_lifetime_source() -> &'static str {
    todo!("a returned string slice must be tied to which source?");
}

fn default_generic_dispatch() -> Dispatch {
    todo!("monomorphized generics usually use which dispatch kind?");
}

fn main() {
    println!("{}", generic_parameter_name());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_generic_parameter_convention() {
        assert_eq!(generic_parameter_name(), 'T');
    }

    #[test]
    fn knows_display_bound() {
        assert_eq!(display_trait_bound(), "Display");
    }

    #[test]
    fn ties_borrowed_returns_to_inputs() {
        assert_eq!(borrowed_return_lifetime_source(), "input reference");
    }

    #[test]
    fn generics_are_statically_dispatched() {
        assert_eq!(default_generic_dispatch(), Dispatch::Static);
        assert_eq!(Dispatch::Dynamic, Dispatch::Dynamic);
    }
}
