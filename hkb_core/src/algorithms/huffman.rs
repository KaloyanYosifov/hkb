use std::collections::HashMap;

use crate::data_structures::Node;

#[derive(Eq, Ord, Debug)]
pub struct HuffmanValue {
    char: char,
    occurance: u64,
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
    pub fn compress(text: &str) -> Node<HuffmanValue> {
        let mut huffman_values: Vec<HuffmanValue> = Vec::with_capacity(text.len() / 2);
        let mut occurance_map: HashMap<char, usize> = HashMap::with_capacity(text.len() / 2);

        for c in text.chars().into_iter() {
            let index = occurance_map
                .entry(c)
                .or_insert_with(|| huffman_values.len() + 1);
            let actual_index = *index - 1;

            if actual_index >= huffman_values.len() {
                huffman_values.push(HuffmanValue {
                    char: c,
                    occurance: 0,
                });
            }

            huffman_values[actual_index].occurance += 1;
        }

        huffman_values.sort();

        println!("{:?}", huffman_values);

        let node = Node::with_value(HuffmanValue {
            char: 'a',
            occurance: 32,
        });

        node
    }
}

#[cfg(test)]
mod tests {
    use super::HuffmanEncoder;

    #[test]
    fn it_can_create_huffman_table_from_text() {
        let text = "Hello there magnificent mothertrucker";

        let node = HuffmanEncoder::compress(text);

        assert!(false);
    }
}
