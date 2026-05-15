// Typing lecture: generics.
//
// A generic type parameter lets one data structure work for many concrete
// types. `Stack<T>` should work as a stack of `i32`, `String`, or any other
// value type without duplicating the implementation.

#[derive(Debug, Default)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn size(&self) -> usize {
        self.items.len()
    }
}

fn main() {
    let mut stack = Stack::new();
    stack.push(10);
    stack.push(20);
    println!("top: {:?}", stack.peek());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_works_for_integers() {
        let mut stack = Stack::new();

        assert!(stack.is_empty());
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.size(), 3);
        assert_eq!(stack.peek(), Some(&3));
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn stack_works_for_strings() {
        let mut stack = Stack::new();

        stack.push(String::from("generics"));
        stack.push(String::from("reuse"));

        assert_eq!(stack.peek().map(String::as_str), Some("reuse"));
        assert_eq!(stack.pop().as_deref(), Some("reuse"));
        assert_eq!(stack.pop().as_deref(), Some("generics"));
    }
}
