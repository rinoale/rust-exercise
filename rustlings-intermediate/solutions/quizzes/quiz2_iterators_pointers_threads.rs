// Quiz 2: closures, iterators, smart pointers, and threads.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PointerTool {
    Box,
    Rc,
    RefCell,
    Arc,
    Mutex,
}

fn iterator_terminal_method() -> &'static str {
    "collect"
}

fn closure_trait_for_mutation() -> &'static str {
    "FnMut"
}

fn choose_pointer_tool(need: &str) -> PointerTool {
    match need {
        "heap allocation" => PointerTool::Box,
        "single-thread shared ownership" => PointerTool::Rc,
        "runtime borrow checks" => PointerTool::RefCell,
        "thread-safe shared ownership" => PointerTool::Arc,
        "thread-safe mutation" => PointerTool::Mutex,
        _ => panic!("unknown pointer need"),
    }
}

fn channel_pattern_name() -> &'static str {
    "message passing"
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
