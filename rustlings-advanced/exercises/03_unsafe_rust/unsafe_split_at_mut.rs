// Typing lecture: safe API around raw pointers.
//
// `split_at_mut` cannot be implemented with normal indexing alone because it
// returns two mutable slices from one input slice. Raw pointers can express the
// disjoint halves, and the public function keeps the unsafe block contained.
//
// TODO: Implement a safe wrapper using `as_mut_ptr` and `from_raw_parts_mut`.

use std::slice;

fn split_at_mut_manual<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    todo!("assert bounds, then create two disjoint mutable slices");
}

fn main() {
    let mut values = [1, 2, 3, 4];
    let (left, right) = split_at_mut_manual(&mut values, 2);
    println!("{left:?} {right:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_and_mutates_both_halves() {
        let mut values = [1, 2, 3, 4, 5, 6];

        let (left, right) = split_at_mut_manual(&mut values, 3);
        left[0] = 10;
        right[0] = 40;

        assert_eq!(values, [10, 2, 3, 40, 5, 6]);
    }

    #[test]
    fn allows_empty_halves() {
        let mut values = [1, 2, 3];

        let (left, right) = split_at_mut_manual(&mut values, 0);
        assert_eq!(left, &mut []);
        assert_eq!(right, &mut [1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn rejects_out_of_bounds_midpoint() {
        let mut values = [1, 2, 3];
        let _ = split_at_mut_manual(&mut values, 4);
    }
}
