use std::collections::VecDeque;

use crate::data_structures::NodeRef;

use super::{HuffmanNode, HuffmanValue};

type HuffmanNodeRef = NodeRef<HuffmanValue>;

fn traverse_node<T, F: Fn(&String, &HuffmanNode) -> Option<T>>(
    node: &HuffmanNode,
    callback: F,
) -> Option<T> {
    let mut queue: VecDeque<(String, HuffmanNodeRef)> = VecDeque::with_capacity(32);

    if let Some(node) = node.get_left() {
        queue.push_front(("0".to_string(), node))
    }

    if let Some(node) = node.get_right() {
        queue.push_front(("1".to_string(), node))
    }

    while !queue.is_empty() {
        let (binary, node) = queue.pop_front().unwrap();
        let borrowed = node.borrow();
        let return_val = callback(&binary, &borrowed);

        if return_val.is_some() {
            return return_val;
        }

        if let Some(node) = borrowed.get_left() {
            queue.push_front((format!("{}0", binary), node))
        }

        if let Some(node) = borrowed.get_right() {
            queue.push_front((format!("{}1", binary), node))
        }
    }

    None
}

pub trait HuffmanNodeTraverser {
    fn to_binary(&self, c: char) -> Option<String>;
    fn from_binary(&self, binary_encoded: String) -> Option<char>;
}

// TODO: Find out if the solutions below are ok for us
// or if we should find a better faster solution
impl HuffmanNodeTraverser for HuffmanNode {
    fn to_binary(&self, c: char) -> Option<String> {
        traverse_node(self, |binary, node| {
            if Some(c) == node.val.char {
                return Some(binary.clone());
            } else {
                None
            }
        })
    }

    fn from_binary(&self, binary_encoded: String) -> Option<char> {
        traverse_node(self, move |binary, node| {
            if binary_encoded.as_str() == binary.as_str() && node.val.char.is_some() {
                return node.val.char;
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, hash};

    use insta::assert_snapshot;

    use crate::algorithms::{HuffmanEncoder, HuffmanNode, HuffmanNodeTraverser, HuffmanValue};

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

        assert_eq!("0", node.to_binary('a').unwrap().as_str());
        assert_eq!("1", node.to_binary('b').unwrap().as_str());
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

        assert_eq!('a', node.from_binary("0".to_string()).unwrap());
        assert_eq!('b', node.from_binary("1".to_string()).unwrap());
        assert!(matches!(node.from_binary("01".to_string()), None));
    }

    #[test]
    fn it_can_convert_char_to_binary_snap() {
        let text = "Hello there magnificent mothertrucker";
        let node = HuffmanEncoder::compress(text);
        let mut output = String::with_capacity(1024);
        let mut chars = text
            .chars()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        chars.sort();

        for c in chars {
            output
                .push_str(format!("Character {} = {} \n", c, node.to_binary(c).unwrap()).as_str());
        }

        assert_snapshot!(output);
    }

    #[test]
    fn it_can_convert_binary_to_char() {
        let text = "Hello there magnificent mothertrucker";
        let node = HuffmanEncoder::compress(text);

        let expected: Vec<(char, &str)> = vec![
            ('H', "01010"),
            ('c', "1011"),
            ('k', "01111"),
            ('n', "11110"),
            ('t', "100"),
            ('l', "11111"),
            ('o', "0110"),
            (' ', "1110"),
            ('r', "001"),
            ('e', "110"),
            ('i', "1010"),
            ('m', "0100"),
            ('u', "01110"),
            ('a', "01011"),
            ('f', "00011"),
            ('g', "00010"),
            ('h', "0000"),
        ];

        for (expected_char, binary_encoded) in expected.into_iter() {
            let char = node.from_binary(binary_encoded.to_string()).unwrap();

            assert_eq!(expected_char, char);
        }
    }
}
