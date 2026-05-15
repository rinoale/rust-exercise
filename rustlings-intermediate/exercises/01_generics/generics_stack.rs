// Typing lecture: generics.
//
// A generic type parameter lets one data structure work for many concrete
// types. `Stack<T>` should work as a stack of `i32`, `String`, or any other
// value type without duplicating the implementation.
//
// TODO: Fill the `todo!()` bodies. The tests at the bottom describe the behavior.

#[derive(Debug, Default)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn push(&mut self, item: T) {
        todo!("push item onto the top of the stack");
    }

    fn pop(&mut self) -> Option<T> {
        todo!("remove and return the top item");
    }

    fn peek(&self) -> Option<&T> {
        todo!("return a shared reference to the top item");
    }

    fn is_empty(&self) -> bool {
        todo!("return true when there are no items");
    }

    fn size(&self) -> usize {
        todo!("return the number of items");
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
