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

#[derive(Eq, Ord, Clone, Copy, Debug)]
pub struct HuffmanBinaryValue {
    pub(crate) val: u32,
    pub(crate) max_bits: u32,
}

impl HuffmanBinaryValue {
    pub fn new(val: u32, max_bits: u32) -> Self {
        Self { val, max_bits }
    }

    pub fn with_value(val: u32) -> Self {
        Self::new(val, 32)
    }

    pub fn with_max_bits(mut self, max_bits: u32) -> Self {
        self.max_bits = max_bits;

        self
    }

    pub fn from_string(value: impl AsRef<str>) -> Self {
        let number = {
            if let Ok(num) = u32::from_str_radix(value.as_ref(), 2) {
                num
            } else {
                0
            }
        };

        Self::new(number, value.as_ref().len() as u32)
    }

    pub fn to_string_packed(&self) -> String {
        format!("{:0width$b}", self.val, width = self.max_bits as usize)
    }

    pub fn to_string(&self) -> String {
        format!("{:032b}", self.val)
    }
}

impl PartialEq for HuffmanBinaryValue {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val && self.max_bits == other.max_bits
    }
}

impl PartialEq<u32> for HuffmanBinaryValue {
    fn eq(&self, other: &u32) -> bool {
        self.val == *other
    }
}

impl PartialEq<HuffmanBinaryValue> for u32 {
    fn eq(&self, other: &HuffmanBinaryValue) -> bool {
        *self == other.val
    }
}

impl PartialOrd for HuffmanBinaryValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let order = {
            if self.val > other.val {
                std::cmp::Ordering::Greater
            } else if self.val < other.val {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        };

        Some(order)
    }
}

impl Into<HuffmanBinaryValue> for u32 {
    fn into(self) -> HuffmanBinaryValue {
        HuffmanBinaryValue::with_value(self)
    }
}

impl std::ops::Add<u32> for HuffmanBinaryValue {
    type Output = HuffmanBinaryValue;

    fn add(self, rhs: u32) -> Self::Output {
        Self::new(self.val + rhs, self.max_bits)
    }
}

impl std::ops::Shl<u32> for HuffmanBinaryValue {
    type Output = HuffmanBinaryValue;

    fn shl(self, rhs: u32) -> Self::Output {
        Self::new(self.val << rhs, self.max_bits)
    }
}

#[derive(Debug)]
pub struct HuffmanBinary {
    pub(crate) binary: Vec<HuffmanBinaryValue>,
}

impl HuffmanBinary {
    pub fn to_string_packed(&self) -> String {
        let mut output = String::with_capacity(self.binary.len());

        for value in self.binary.iter() {
            output.push_str(value.to_string_packed().as_str());
        }

        output
    }
}

impl ToString for HuffmanBinary {
    fn to_string(&self) -> String {
        let mut output = String::with_capacity(self.binary.len());

        for value in self.binary.iter() {
            output.push_str(value.to_string().as_str());
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::HuffmanBinaryValue;

    use super::HuffmanBinary;

    #[test]
    fn it_can_convert_binary_into_string() {
        let binary = HuffmanBinary {
            binary: vec![112.into(), 5.into(), 32.into()],
        };

        assert_eq!("000000000000000000000000011100000000000000000000000000000000010100000000000000000000000000100000", binary.to_string());
    }

    #[test]
    fn it_can_convert_binary_into_string_packed() {
        let binary = HuffmanBinary {
            binary: vec![
                HuffmanBinaryValue::new(112, 8),
                HuffmanBinaryValue::new(5, 3),
                HuffmanBinaryValue::new(32, 5),
            ],
        };

        assert_eq!("01110000101100000", binary.to_string_packed());
    }
}
