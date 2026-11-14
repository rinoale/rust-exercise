// Typing lecture: safe API around raw pointers.

use std::slice;

fn split_at_mut_manual<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= slice.len(), "midpoint out of bounds");

    let len = slice.len();
    let ptr = slice.as_mut_ptr();

    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
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
