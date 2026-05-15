// Typing lecture: concurrency.
//
// Threads need owned data. Channels let worker threads send partial results
// back to the main thread. This is the same high-level shape used in larger
// map-reduce style jobs.
//
// TODO: Spawn workers, receive partial maps, and merge the word counts.

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

fn normalize(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn parallel_word_count(text: &str, worker_count: usize) -> HashMap<String, usize> {
    todo!("split words across threads, receive partial maps, and merge them");
}

fn main() {
    let counts = parallel_word_count("Rust makes fearless concurrency practical", 2);
    println!("{counts:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words_across_workers() {
        let text = "Rust rust ownership threads ownership channels rust";
        let counts = parallel_word_count(text, 3);

        assert_eq!(counts.get("rust"), Some(&3));
        assert_eq!(counts.get("ownership"), Some(&2));
        assert_eq!(counts.get("threads"), Some(&1));
        assert_eq!(counts.get("channels"), Some(&1));
    }

    #[test]
    fn normalizes_case_and_punctuation() {
        let counts = parallel_word_count("Safe, safe. SAFE!", 4);

        assert_eq!(counts.get("safe"), Some(&3));
        assert_eq!(counts.len(), 1);
    }

    #[test]
    fn empty_input_returns_empty_map() {
        assert!(parallel_word_count("", 4).is_empty());
    }
}
