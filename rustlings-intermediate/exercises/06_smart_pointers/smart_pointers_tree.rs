// Typing lecture: smart pointers.
//
// `Rc<T>` enables shared ownership in one thread. `RefCell<T>` enables
// interior mutability checked at runtime. Together, `Rc<RefCell<T>>` is useful
// for tree-shaped teaching examples where many handles point at the same node.
//
// TODO: Mutate child lists and walk the tree through smart pointer handles.

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
        todo!("borrow the parent mutably and push the child");
    }

    fn total_nodes(node: &NodeRef) -> usize {
        todo!("count this node and all descendants");
    }

    fn render(node: &NodeRef) -> String {
        todo!("render one value per line with two spaces per level");
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
