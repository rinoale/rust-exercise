// Quiz 1: advanced traits and lifetimes.
//
// TODO: Answer the concept questions by returning the expected values.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchKind {
    Static,
    Dynamic,
}

fn associated_type_purpose() -> &'static str {
    todo!("what does an associated type name inside a trait?");
}

fn trait_object_dispatch() -> DispatchKind {
    todo!("what dispatch kind does dyn Trait use?");
}

fn hrtb_meaning() -> &'static str {
    todo!("what does for<'a> express?");
}

fn main() {
    println!("{}", associated_type_purpose());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associated_types_name_outputs() {
        assert_eq!(associated_type_purpose(), "output type");
    }

    #[test]
    fn dyn_trait_is_dynamic_dispatch() {
        assert_eq!(trait_object_dispatch(), DispatchKind::Dynamic);
    }

    #[test]
    fn generic_traits_can_use_static_dispatch() {
        assert_eq!(DispatchKind::Static, DispatchKind::Static);
    }

    #[test]
    fn hrtb_works_for_any_lifetime() {
        assert_eq!(hrtb_meaning(), "works for any lifetime");
    }
}
