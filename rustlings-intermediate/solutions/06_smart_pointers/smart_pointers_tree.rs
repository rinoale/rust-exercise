// Typing lecture: smart pointers.

use std::cell::RefCell;
use std::rc::Rc;

type NodeRef = Rc<RefCell<TreeNode>>;

#[derive(Debug)]
struct TreeNode {
    value: i32,
    children: Vec<NodeRef>,
}

impl TreeNode {
    fn new(value: i32) -> NodeRef {
        Rc::new(RefCell::new(Self {
            value,
            children: Vec::new(),
        }))
    }

    fn add_child(parent: &NodeRef, child: NodeRef) {
        parent.borrow_mut().children.push(child);
    }

    fn total_nodes(node: &NodeRef) -> usize {
        let node = node.borrow();
        1 + node.children.iter().map(Self::total_nodes).sum::<usize>()
    }

    fn render(node: &NodeRef) -> String {
        fn render_inner(node: &NodeRef, depth: usize, output: &mut String) {
            let node = node.borrow();
            output.push_str(&"  ".repeat(depth));
            output.push_str(&node.value.to_string());
            output.push('\n');

            for child in &node.children {
                render_inner(child, depth + 1, output);
            }
        }

        let mut output = String::new();
        render_inner(node, 0, &mut output);
        output
    }
}

fn main() {
    let root = TreeNode::new(1);
    TreeNode::add_child(&root, TreeNode::new(2));
    println!("{}", TreeNode::render(&root));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> NodeRef {
        let root = TreeNode::new(1);
        let left = TreeNode::new(2);
        let right = TreeNode::new(3);
        let leaf = TreeNode::new(4);

        TreeNode::add_child(&left, leaf);
        TreeNode::add_child(&root, left);
        TreeNode::add_child(&root, right);

        root
    }

    #[test]
    fn counts_nodes_recursively() {
        let root = sample_tree();

        assert_eq!(TreeNode::total_nodes(&root), 4);
    }

    #[test]
    fn renders_indented_tree() {
        let root = sample_tree();

        assert_eq!(TreeNode::render(&root), "1\n  2\n    4\n  3\n");
    }
}
