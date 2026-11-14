// Typing lecture: lifetime relationships.

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
        &self.source[self.start..self.end]
    }

    fn before(&self) -> &'a str {
        &self.source[..self.start]
    }
}

fn choose_longer<'a>(left: &'a str, right: &'a str) -> &'a str {
    if left.len() > right.len() {
        left
    } else {
        right
    }
}

fn prefix_before_marker<'a>(source: &'a str, marker: &str) -> &'a str {
    match source.find(marker) {
        Some(index) => &source[..index],
        None => source,
    }
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
