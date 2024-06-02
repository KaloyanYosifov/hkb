use std::collections::HashMap;
pub use traits::*;

mod traits;

use crate::data_structures::{MinHeap, Node};

pub type HuffmanNode = Node<HuffmanValue>;

#[derive(Eq, Ord, Clone, Copy, Debug)]
pub struct HuffmanValue {
    pub char: Option<char>,
    pub occurance: u64,
}

impl PartialEq for HuffmanValue {
    fn eq(&self, other: &Self) -> bool {
        self.char == other.char
    }
}

impl PartialOrd for HuffmanValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let val = {
            if self.occurance > other.occurance {
                std::cmp::Ordering::Greater
            } else if self.occurance < other.occurance {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        };

        Some(val)
    }
}

pub struct HuffmanEncoder;

impl HuffmanEncoder {
    pub fn compress(text: &str) -> HuffmanNode {
        let mut huffman_values: Vec<HuffmanValue> = Vec::with_capacity(text.len() / 2);
        let mut occurance_map: HashMap<char, usize> = HashMap::with_capacity(text.len() / 2);

        for c in text.chars().into_iter() {
            let index = occurance_map
                .entry(c)
                .or_insert_with(|| huffman_values.len() + 1);
            let actual_index = *index - 1;

            if actual_index >= huffman_values.len() {
                huffman_values.push(HuffmanValue {
                    char: Some(c),
                    occurance: 0,
                });
            }

            huffman_values[actual_index].occurance += 1;
        }

        // TODO: Check if we can do this more efficiently without using an intermediate vector
        // and if we can just directly push to the min heap and store a pointer to a map to update
        // the struct whenever we need to
        let mut priority_queue: MinHeap<HuffmanNode> = MinHeap::with_capacity(text.len() / 2);
        for value in huffman_values.into_iter() {
            priority_queue.insert(HuffmanNode::with_value(value));
        }

        while priority_queue.size() > 1 {
            let el1 = priority_queue.pop().unwrap();
            let el2 = priority_queue.pop().unwrap();

            let occurance = el1.val.occurance + el2.val.occurance;
            let node = HuffmanNode::with_nodes(
                HuffmanValue {
                    char: None,
                    occurance,
                },
                el1,
                el2,
            );

            priority_queue.insert(node);
        }

        priority_queue.pop().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::HuffmanEncoder;

    #[test]
    fn it_generates_correct_snapshot() {
        let text = "Hello there magnificent mothertrucker";

        assert_debug_snapshot!(HuffmanEncoder::compress(text));
    }
}
