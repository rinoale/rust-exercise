// Typing lecture: concurrency.

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

fn normalize(word: &str) -> String {
    word.trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

fn parallel_word_count(text: &str, worker_count: usize) -> HashMap<String, usize> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(normalize)
        .filter(|word| !word.is_empty())
        .collect();

    if words.is_empty() {
        return HashMap::new();
    }

    let worker_count = worker_count.max(1).min(words.len());
    let chunk_size = words.len().div_ceil(worker_count);
    let (tx, rx) = mpsc::channel();

    for chunk in words.chunks(chunk_size) {
        let tx = tx.clone();
        let chunk = chunk.to_vec();

        thread::spawn(move || {
            let mut counts = HashMap::new();

            for word in chunk {
                *counts.entry(word).or_insert(0) += 1;
            }

            tx.send(counts).expect("receiver should still be alive");
        });
    }

    drop(tx);

    let mut merged = HashMap::new();
    for partial in rx {
        for (word, count) in partial {
            *merged.entry(word).or_insert(0) += count;
        }
    }

    merged
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
