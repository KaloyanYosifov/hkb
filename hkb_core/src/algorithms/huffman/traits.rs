use std::collections::VecDeque;

use crate::data_structures::NodeRef;

use super::{HuffmanBinaryValue, HuffmanNode, HuffmanValue};

type HuffmanNodeRef = NodeRef<HuffmanValue>;

fn traverse_node<T, F: Fn(HuffmanBinaryValue, &HuffmanNode) -> Option<T>>(
    node: &HuffmanNode,
    callback: F,
) -> Option<T> {
    let mut queue: VecDeque<(HuffmanBinaryValue, HuffmanNodeRef)> = VecDeque::with_capacity(32);

    if let Some(node) = node.get_left() {
        queue.push_front((HuffmanBinaryValue::new(0, 1), node));
    }

    if let Some(node) = node.get_right() {
        queue.push_front((HuffmanBinaryValue::new(1, 1), node));
    }

    while !queue.is_empty() {
        let (binary, node) = queue.pop_front().unwrap();
        let borrowed = node.borrow();
        let return_val = callback(binary, &borrowed);

        if return_val.is_some() {
            return return_val;
        }

        if let Some(node) = borrowed.get_left() {
            let new_value = (binary << 1).with_max_bits(binary.max_bits + 1);
            queue.push_front((new_value, node));
        }

        if let Some(node) = borrowed.get_right() {
            let new_value = ((binary << 1) + 1).with_max_bits(binary.max_bits + 1);
            queue.push_front((new_value, node));
        }
    }

    None
}

pub trait HuffmanNodeTraverser {
    fn to_binary(&self, c: char) -> Option<HuffmanBinaryValue>;
    fn from_binary(&self, binary_encoded: HuffmanBinaryValue) -> Option<char>;
}

// TODO: Find out if the solutions below are ok for us
// or if we should find a better faster solution
impl HuffmanNodeTraverser for HuffmanNode {
    fn to_binary(&self, c: char) -> Option<HuffmanBinaryValue> {
        traverse_node(self, |binary, node| {
            if Some(c) == node.val.char {
                return Some(binary);
            } else {
                None
            }
        })
    }

    fn from_binary(&self, binary_encoded: HuffmanBinaryValue) -> Option<char> {
        traverse_node(self, move |binary, node| {
            if binary_encoded == binary && node.val.char.is_some() {
                return node.val.char;
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::algorithms::Huffman;

    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn it_can_convert_char_to_binary() {
        let node = HuffmanNode::new(
            HuffmanValue {
                char: None,
                occurance: 2,
            },
            Some(HuffmanValue {
                char: Some('a'),
                occurance: 1,
            }),
            Some(HuffmanValue {
                char: Some('b'),
                occurance: 1,
            }),
        );

        assert_eq!(0, node.to_binary('a').unwrap());
        assert_eq!(1, node.to_binary('b').unwrap());
        assert!(matches!(node.to_binary('c'), None));
    }

    #[test]
    fn it_can_convert_from_binary_to_char() {
        let node = HuffmanNode::new(
            HuffmanValue {
                char: None,
                occurance: 2,
            },
            Some(HuffmanValue {
                char: Some('a'),
                occurance: 1,
            }),
            Some(HuffmanValue {
                char: Some('b'),
                occurance: 1,
            }),
        );

        assert_eq!(
            'a',
            node.from_binary(HuffmanBinaryValue::from_string("0"))
                .unwrap()
        );
        assert_eq!(
            'b',
            node.from_binary(HuffmanBinaryValue::from_string("1"))
                .unwrap()
        );
        assert!(matches!(
            node.from_binary(HuffmanBinaryValue::from_string("01")),
            None
        ));
    }

    #[test]
    fn it_can_convert_char_to_binary_snap() {
        let text = "Hello there magnificent mothertrucker";
        let (_, node) = Huffman::encode(text);
        let mut output = String::with_capacity(1024);
        let mut chars = text
            .chars()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        chars.sort();

        for c in chars {
            output.push_str(
                format!(
                    "Character {} = {} \n",
                    c,
                    node.to_binary(c).unwrap().to_string_packed()
                )
                .as_str(),
            );
        }

        assert_snapshot!(output);
    }

    #[test]
    fn it_can_convert_binary_to_char() {
        let text = "Hello there magnificent mothertrucker";
        let (_, node) = Huffman::encode(text);

        let expected: Vec<(char, HuffmanBinaryValue)> = vec![
            ('H', HuffmanBinaryValue::from_string("01010")),
            ('c', HuffmanBinaryValue::from_string("1011")),
            ('k', HuffmanBinaryValue::from_string("01111")),
            ('n', HuffmanBinaryValue::from_string("11110")),
            ('t', HuffmanBinaryValue::from_string("100")),
            ('l', HuffmanBinaryValue::from_string("11111")),
            ('o', HuffmanBinaryValue::from_string("0110")),
            (' ', HuffmanBinaryValue::from_string("1110")),
            ('r', HuffmanBinaryValue::from_string("001")),
            ('e', HuffmanBinaryValue::from_string("110")),
            ('i', HuffmanBinaryValue::from_string("1010")),
            ('m', HuffmanBinaryValue::from_string("0100")),
            ('u', HuffmanBinaryValue::from_string("01110")),
            ('a', HuffmanBinaryValue::from_string("01011")),
            ('f', HuffmanBinaryValue::from_string("00011")),
            ('g', HuffmanBinaryValue::from_string("00010")),
            ('h', HuffmanBinaryValue::from_string("0000")),
        ];

        for (expected_char, binary_encoded) in expected.into_iter() {
            println!("{} - {}", expected_char, binary_encoded.val);
            let char = node.from_binary(binary_encoded).unwrap();

            assert_eq!(expected_char, char);
        }
    }
}
