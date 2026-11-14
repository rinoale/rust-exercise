// Typing lecture: `Cow<'a, str>`.

use std::borrow::Cow;

fn normalize_tag(input: &str) -> Cow<'_, str> {
    let trimmed = input.trim();
    let already_normalized = trimmed == input
        && input
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');

    if already_normalized {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(trimmed.to_ascii_lowercase().replace(' ', "_"))
    }
}

fn ensure_suffix<'a>(input: Cow<'a, str>, suffix: &str) -> Cow<'a, str> {
    if input.ends_with(suffix) {
        input
    } else {
        Cow::Owned(format!("{input}{suffix}"))
    }
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
