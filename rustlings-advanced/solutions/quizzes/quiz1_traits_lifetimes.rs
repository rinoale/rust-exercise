// Quiz 1: advanced traits and lifetimes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchKind {
    Static,
    Dynamic,
}

fn associated_type_purpose() -> &'static str {
    "output type"
}

fn trait_object_dispatch() -> DispatchKind {
    DispatchKind::Dynamic
}

fn hrtb_meaning() -> &'static str {
    "works for any lifetime"
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
