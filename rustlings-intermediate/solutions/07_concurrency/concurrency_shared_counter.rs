// Typing lecture: shared state with `Arc<Mutex<T>>`.

use std::sync::{Arc, Mutex};
use std::thread;

fn sum_in_threads(chunks: Vec<Vec<u32>>) -> u32 {
    let total = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for chunk in chunks {
        let total = Arc::clone(&total);

        let handle = thread::spawn(move || {
            let partial: u32 = chunk.iter().sum();
            let mut guard = total.lock().unwrap();
            *guard += partial;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let total = total.lock().unwrap();
    *total
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
