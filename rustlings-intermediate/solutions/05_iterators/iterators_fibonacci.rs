// Typing lecture: iterators.

#[derive(Debug, Clone)]
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Self {
            current: 0,
            next: 1,
        }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.current;
        self.current = self.next;
        self.next += value;
        Some(value)
    }
}

fn sum_even_fibonacci_from_first(count: usize) -> u64 {
    Fibonacci::new().take(count).filter(|n| n % 2 == 0).sum()
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
