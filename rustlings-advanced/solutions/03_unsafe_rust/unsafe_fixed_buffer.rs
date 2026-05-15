// Typing lecture: unsafe Rust.

use std::mem::MaybeUninit;

struct FixedBuffer<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> FixedBuffer<T, N> {
    fn new() -> Self {
        Self {
            storage: std::array::from_fn(|_| MaybeUninit::uninit()),
            head: 0,
            len: 0,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_full(&self) -> bool {
        self.len == N
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        let index = (self.head + self.len) % N;
        self.storage[index].write(value);
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let index = self.head;
        self.head = (self.head + 1) % N;
        self.len -= 1;

        // SAFETY: The invariant says every slot in the initialized range is
        // initialized. `index` is the oldest initialized slot and we decrement
        // `len`, so it will not be read or dropped again.
        Some(unsafe { self.storage[index].assume_init_read() })
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
