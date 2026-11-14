// Typing lecture: `Cow<'a, str>`.
//
// `Cow` means clone-on-write. It can hold borrowed data when no change is
// needed, or owned data when normalization requires allocation.
//
// TODO: Return borrowed values when possible and owned values when necessary.

use std::borrow::Cow;

fn normalize_tag(input: &str) -> Cow<'_, str> {
    todo!("trim, lowercase, and replace spaces only when needed");
}

fn ensure_suffix<'a>(input: Cow<'a, str>, suffix: &str) -> Cow<'a, str> {
    todo!("reuse the input when it already ends with suffix");
}

fn main() {
    println!("{}", normalize_tag(" Rust Lang "));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrows_when_normalization_is_not_needed() {
        let normalized = normalize_tag("rust_lang");

        assert!(matches!(normalized, Cow::Borrowed("rust_lang")));
    }

    #[test]
    fn owns_when_normalization_changes_text() {
        let normalized = normalize_tag(" Rust Lang ");

        assert!(matches!(normalized, Cow::Owned(_)));
        assert_eq!(normalized, "rust_lang");
    }

    #[test]
    fn reuses_existing_suffix() {
        let value = Cow::Borrowed("report.md");
        let with_suffix = ensure_suffix(value, ".md");

        assert!(matches!(with_suffix, Cow::Borrowed("report.md")));
    }

    #[test]
    fn allocates_when_suffix_is_missing() {
        let value = Cow::Borrowed("report");
        let with_suffix = ensure_suffix(value, ".md");

        assert!(matches!(with_suffix, Cow::Owned(_)));
        assert_eq!(with_suffix, "report.md");
    }
}
