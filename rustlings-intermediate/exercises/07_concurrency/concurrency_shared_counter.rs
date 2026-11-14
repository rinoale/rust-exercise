// Typing lecture: shared state with `Arc<Mutex<T>>`.
//
// Threads need owned data. `Arc<T>` gives shared ownership across threads and
// `Mutex<T>` gives one mutable access at a time.
//
// TODO: Spawn workers, update the shared total, and join every thread.

use std::sync::{Arc, Mutex};
use std::thread;

fn sum_in_threads(chunks: Vec<Vec<u32>>) -> u32 {
    todo!("sum chunks on worker threads into an Arc<Mutex<u32>>");
}

fn main() {
    let total = sum_in_threads(vec![vec![1, 2, 3], vec![4, 5]]);
    println!("{total}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_chunks_across_threads() {
        let chunks = vec![vec![1, 2, 3], vec![10, 20], vec![100]];

        assert_eq!(sum_in_threads(chunks), 136);
    }

    #[test]
    fn handles_empty_work() {
        assert_eq!(sum_in_threads(Vec::new()), 0);
        assert_eq!(sum_in_threads(vec![Vec::new(), vec![5]]), 5);
    }
}
