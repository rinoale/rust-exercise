// Typing lecture: iterators.
//
// Implementing `Iterator` requires one method: `next`. Iterator adaptors such
// as `take`, `filter`, and `sum` then become available for free.
//
// TODO: Complete the iterator state transitions and adaptor chain.

#[derive(Debug, Clone)]
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        todo!("start at 0, then 1");
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("yield the current number and advance the state");
    }
}

fn sum_even_fibonacci_from_first(count: usize) -> u64 {
    todo!("take the first count Fibonacci numbers, keep evens, and sum them");
}

fn main() {
    let first_ten: Vec<_> = Fibonacci::new().take(10).collect();
    println!("{first_ten:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yields_fibonacci_sequence() {
        let values: Vec<_> = Fibonacci::new().take(10).collect();

        assert_eq!(values, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn chains_iterator_adaptors() {
        assert_eq!(sum_even_fibonacci_from_first(20), 3382);
    }
}
