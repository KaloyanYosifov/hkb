use std::{cell::RefCell, rc::Rc};

pub trait Constraints: PartialEq + Eq + PartialOrd + Ord {}

impl<T: PartialEq + Eq + PartialOrd + Ord> Constraints for T {}

pub type NodeRef<T> = Rc<RefCell<Node<T>>>;

#[derive(Eq, Debug)]
pub struct Node<T: PartialEq + Eq + PartialOrd + Ord> {
    pub val: T,
    left: Option<NodeRef<T>>,
    right: Option<NodeRef<T>>,
}

impl<T: Constraints> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<T: Constraints> Ord for Node<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.val > other.val {
            std::cmp::Ordering::Greater
        } else if self.val < other.val {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl<T: Constraints> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Constraints> Node<T> {
    pub fn new(val: T, left_val: Option<T>, right_val: Option<T>) -> Self {
        let left = { left_val.map(|v| Rc::new(RefCell::new(Node::with_value(v)))) };
        let right = { right_val.map(|v| Rc::new(RefCell::new(Node::with_value(v)))) };

        Self { val, left, right }
    }

    pub fn with_value(val: T) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }

    pub fn with_nodes(val: T, left: Node<T>, right: Node<T>) -> Self {
        Self {
            val,
            left: Some(Rc::new(RefCell::new(left))),
            right: Some(Rc::new(RefCell::new(right))),
        }
    }

    pub fn get_left(&self) -> Option<NodeRef<T>> {
        self.left.as_ref().map(|node| node.clone())
    }

    pub fn get_right(&self) -> Option<NodeRef<T>> {
        self.right.as_ref().map(|node| node.clone())
    }

    pub fn height(&self) -> usize {
        self.height_recursive(self)
    }

    fn height_recursive(&self, node: &Node<T>) -> usize {
        match (node.get_left(), node.get_right()) {
            (Some(left), Some(right)) => {
                let left_height = self.height_recursive(&left.borrow());
                let right_height = self.height_recursive(&right.borrow());
                1 + std::cmp::max(left_height, right_height)
            }
            (Some(left), None) => 1 + self.height_recursive(&left.borrow()),
            (None, Some(right)) => 1 + self.height_recursive(&right.borrow()),
            (None, None) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn it_can_create_a_node_with_value() {
        let node = Node::with_value(3);

        assert_eq!(3, node.val);
        assert!(node.get_left().is_none());
        assert!(node.get_right().is_none());
    }

    #[test]
    fn it_can_create_a_node_with_left_right_moved() {
        let node = Node::new(3, Some(5), Some(7));

        assert_eq!(3, node.val);
        assert_eq!(5, node.get_left().unwrap().borrow().val);
        assert_eq!(7, node.get_right().unwrap().borrow().val);
    }

    #[test]
    fn it_can_create_with_nodes_moved_directly() {
        let left_node = Node::with_value(5);
        let right_node = Node::with_value(7);
        let node = Node::with_nodes(3, left_node, right_node);

        assert_eq!(3, node.val);
        assert_eq!(5, node.get_left().unwrap().borrow().val);
        assert_eq!(7, node.get_right().unwrap().borrow().val);
    }

    #[test]
    fn it_can_return_the_height_of_the_node_when_it_is_one() {
        let node = Node::with_value(3);

        assert_eq!(1, node.height());
    }

    #[test]
    fn it_can_return_the_height_of_the_node() {
        let left_node = Node::with_value(5);
        let right_node = Node::with_value(7);
        let node = Node::with_nodes(3, left_node, right_node);

        assert_eq!(2, node.height());
    }
}
