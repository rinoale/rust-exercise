// Typing lecture: unsafe Rust.
//
// This fixed ring buffer uses unsafe internals but exposes a safe public API.
// The invariant is: exactly `len` slots starting at `head` are initialized.
//
// TODO: Use `MaybeUninit` and raw initialization carefully.

use std::mem::MaybeUninit;

struct FixedBuffer<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    fn new() -> Self {
        todo!("create uninitialized storage with head=0 and len=0");
    }

    fn len(&self) -> usize {
        todo!("return the number of initialized items");
    }

    fn is_full(&self) -> bool {
        todo!("return true when len equals capacity");
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        todo!("write into the next free slot or return the value when full");
    }

    fn pop(&mut self) -> Option<T> {
        todo!("read and remove the oldest initialized value");
    }
}

impl<T, const N: usize> Drop for FixedBuffer<T, N> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

fn main() {
    let mut buffer = FixedBuffer::<i32, 2>::new();
    let _ = buffer.push(10);
    println!("len={}", buffer.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_and_pops_in_order() {
        let mut buffer = FixedBuffer::<String, 3>::new();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.push(String::from("a")).is_ok());
        assert!(buffer.push(String::from("b")).is_ok());
        assert!(buffer.push(String::from("c")).is_ok());
        assert!(buffer.is_full());
        assert_eq!(buffer.push(String::from("d")), Err(String::from("d")));

        assert_eq!(buffer.pop().as_deref(), Some("a"));
        assert_eq!(buffer.pop().as_deref(), Some("b"));
        assert_eq!(buffer.pop().as_deref(), Some("c"));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn wraps_around_after_pop() {
        let mut buffer = FixedBuffer::<i32, 2>::new();

        assert_eq!(buffer.push(1), Ok(()));
        assert_eq!(buffer.push(2), Ok(()));
        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.push(3), Ok(()));

        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
    }
}
