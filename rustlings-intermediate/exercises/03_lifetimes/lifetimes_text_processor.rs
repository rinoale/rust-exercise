// Typing lecture: lifetimes.
//
// `TextProcessor<'a>` borrows text owned by someone else. The lifetime
// parameter says every returned substring must not outlive the original text.
//
// TODO: Fill the methods while keeping the borrowed data model.

#[derive(Debug)]
struct TextProcessor<'a> {
    text: &'a str,
}

impl<'a> TextProcessor<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn word_count(&self) -> usize {
        todo!("count whitespace-separated words");
    }

    fn find_word(&self, word: &str) -> Option<usize> {
        todo!("find the byte index of a word");
    }

    fn substring(&self, start: usize, end: usize) -> &'a str {
        todo!("return a slice tied to the original text lifetime");
    }
}

fn main() {
    let text = String::from("lifetimes connect borrows");
    let processor = TextProcessor::new(&text);
    println!("{} words", processor.word_count());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words() {
        let processor = TextProcessor::new("borrowed text stays valid");

        assert_eq!(processor.word_count(), 4);
    }

    #[test]
    fn finds_word_offsets() {
        let processor = TextProcessor::new("zero cost abstractions");

        assert_eq!(processor.find_word("cost"), Some(5));
        assert_eq!(processor.find_word("missing"), None);
    }

    #[test]
    fn returns_borrowed_substring() {
        let text = String::from("lifetime annotations");
        let processor = TextProcessor::new(&text);

        assert_eq!(processor.substring(0, 8), "lifetime");
        assert_eq!(processor.substring(9, 20), "annotations");
    }
}
