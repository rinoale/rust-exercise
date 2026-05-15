// Quiz 2: closures, iterators, smart pointers, and threads.
//
// TODO: Fill in the concept answers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerTool {
    Box,
    Rc,
    RefCell,
    Arc,
    Mutex,
}

fn iterator_terminal_method() -> &'static str {
    todo!("which method consumes an iterator into a collection?");
}

fn closure_trait_for_mutation() -> &'static str {
    todo!("which closure trait allows mutable captures?");
}

fn choose_pointer_tool(need: &str) -> PointerTool {
    todo!("choose the pointer/concurrency tool for the need");
}

fn channel_pattern_name() -> &'static str {
    todo!("what concurrency style sends data between threads?");
}

fn main() {
    println!("{}", iterator_terminal_method());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_iterator_consumers() {
        assert_eq!(iterator_terminal_method(), "collect");
    }

    #[test]
    fn knows_mutating_closure_trait() {
        assert_eq!(closure_trait_for_mutation(), "FnMut");
    }

    #[test]
    fn chooses_pointer_tools() {
        assert_eq!(choose_pointer_tool("heap allocation"), PointerTool::Box);
        assert_eq!(
            choose_pointer_tool("single-thread shared ownership"),
            PointerTool::Rc
        );
        assert_eq!(
            choose_pointer_tool("runtime borrow checks"),
            PointerTool::RefCell
        );
        assert_eq!(
            choose_pointer_tool("thread-safe shared ownership"),
            PointerTool::Arc
        );
        assert_eq!(
            choose_pointer_tool("thread-safe mutation"),
            PointerTool::Mutex
        );
    }

    #[test]
    fn names_message_passing() {
        assert_eq!(channel_pattern_name(), "message passing");
    }
}
