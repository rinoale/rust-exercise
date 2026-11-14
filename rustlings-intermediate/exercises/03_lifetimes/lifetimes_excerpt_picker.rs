// Typing lecture: lifetime relationships.
//
// Lifetimes are most useful when a returned reference is tied to one or more
// input references. This exercise practices borrowed structs, one-lifetime
// return values, and inputs whose lifetimes are intentionally independent.
//
// TODO: Return slices without cloning or allocating.

#[derive(Debug, PartialEq, Eq)]
struct Excerpt<'a> {
    source: &'a str,
    start: usize,
    end: usize,
}

impl<'a> Excerpt<'a> {
    fn new(source: &'a str, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    fn text(&self) -> &'a str {
        todo!("return the selected slice from the original source");
    }

    fn before(&self) -> &'a str {
        todo!("return the source text before the excerpt");
    }
}

fn choose_longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    todo!("return the longer borrowed string slice");
}

fn prefix_before_marker<'a>(source: &'a str, marker: &str) -> &'a str {
    todo!("return the prefix before marker, or the whole source if missing");
}

fn main() {
    let source = String::from("lifetimes connect references");
    let excerpt = Excerpt::new(&source, 0, 9);
    println!("{}", excerpt.text());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_excerpt_slices() {
        let source = String::from("borrowed views avoid copies");
        let excerpt = Excerpt::new(&source, 9, 14);

        assert_eq!(excerpt.text(), "views");
        assert_eq!(excerpt.before(), "borrowed ");
    }

    #[test]
    fn chooses_longer_input_reference() {
        let short = String::from("tiny");
        let long = String::from("much longer");

        assert_eq!(choose_longer(&short, &long), "much longer");
        assert_eq!(choose_longer("same", "size"), "size");
    }

    #[test]
    fn marker_lifetime_is_independent() {
        let source = String::from("title:body text");

        let prefix = {
            let marker = String::from(":");
            prefix_before_marker(&source, &marker)
        };

        assert_eq!(prefix, "title");
    }
}
