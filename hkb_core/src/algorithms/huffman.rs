use std::collections::HashMap;
pub use structs::*;
pub use traits::*;

mod structs;
mod traits;

use crate::data_structures::{MinHeap, Node};

pub type HuffmanNode = Node<HuffmanValue>;

pub struct HuffmanEncoder;

impl HuffmanEncoder {
    pub fn compress(text: &str) -> (HuffmanBinary, HuffmanNode) {
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

        let root = priority_queue.pop().unwrap();

        let mut binary = Vec::with_capacity(text.len());
        for char in text.chars().into_iter() {
            binary.push(root.to_binary(char).unwrap());
        }

        let binary = HuffmanBinary { binary };

        (binary, root)
    }
}

pub struct HuffmanDecoder;
impl HuffmanDecoder {
    pub fn decode(binary: HuffmanBinary, node: HuffmanNode) -> String {
        // TODO: figure out the length to put as a capacity
        let mut message = String::with_capacity(512);

        for byte in binary.binary {
            // TODO: catch errors
            message.push(node.from_binary(byte).unwrap());
        }

        message
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::{HuffmanDecoder, HuffmanEncoder};

    #[test]
    fn it_generates_correct_snapshot() {
        let text = "Hello there magnificent mothertrucker";

        assert_debug_snapshot!(HuffmanEncoder::compress(text));
    }

    #[test]
    fn it_can_decode_huffman_codes() {
        let text = "Hello there magnificent mothertrucker";
        let (binary, node) = HuffmanEncoder::compress(text);

        let message = HuffmanDecoder::decode(binary, node);

        assert_eq!(text, message);
    }
}
