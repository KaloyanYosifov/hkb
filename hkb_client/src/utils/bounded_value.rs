use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
};

pub type BoundValueType = usize;
pub struct BoundedValue {
    val: BoundValueType,
    min_val: BoundValueType,
    max_val: BoundValueType,
}

impl BoundedValue {
    pub fn new(val: BoundValueType, min_val: BoundValueType, max_val: BoundValueType) -> Self {
        Self {
            val,
            min_val,
            max_val,
        }
    }

    pub fn set_val(&mut self, val: BoundValueType) {
        if val > self.max_val {
            self.val = self.max_val;
        } else {
            self.val = val;
        }
    }

    pub fn add_val(&mut self, val: BoundValueType) {
        let mut val = self.val.checked_add(val).unwrap_or(self.max_val);

        if val > self.max_val {
            val = self.max_val;
        }

        self.val = val;
    }

    pub fn sub_val(&mut self, val: BoundValueType) {
        let mut val = self.val.checked_sub(val).unwrap_or(self.max_val);

        if val < self.min_val {
            val = self.min_val;
        }

        self.val = val;
    }

    pub fn set_max(&mut self, max_val: BoundValueType) {
        if self.val > max_val {
            self.val = max_val;
        }

        self.max_val = max_val;
    }

    pub fn set_to_min(&mut self) {
        self.val = self.min_val;
    }

    pub fn set_to_max(&mut self) {
        self.val = self.max_val;
    }

    pub fn get_val(&self) -> BoundValueType {
        self.val
    }

    pub fn lt(&self, val: BoundValueType) -> bool {
        self.val < val
    }

    pub fn gt(&self, val: BoundValueType) -> bool {
        self.val > val
    }

    pub fn eq(&self, val: BoundValueType) -> bool {
        self.val == val
    }
}

impl Add<BoundValueType> for BoundedValue {
    type Output = Self;

    fn add(self, rhs: BoundValueType) -> Self::Output {
        Self {
            min_val: self.min_val,
            max_val: self.max_val,
            val: self.val.checked_add(rhs).unwrap_or(self.max_val),
        }
    }
}

impl Add<u16> for BoundedValue {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        Self {
            min_val: self.min_val,
            max_val: self.max_val,
            val: self.val.checked_add(rhs as usize).unwrap_or(self.max_val),
        }
    }
}

impl AddAssign<BoundValueType> for BoundedValue {
    fn add_assign(&mut self, rhs: BoundValueType) {
        self.add_val(rhs);
    }
}

impl Sub<BoundValueType> for BoundedValue {
    type Output = Self;

    fn sub(self, rhs: BoundValueType) -> Self::Output {
        Self {
            min_val: self.min_val,
            max_val: self.max_val,
            val: self.val.checked_sub(rhs).unwrap_or(self.min_val),
        }
    }
}

impl Sub<u16> for BoundedValue {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
        Self {
            min_val: self.min_val,
            max_val: self.max_val,
            val: self.val.checked_sub(rhs as usize).unwrap_or(self.min_val),
        }
    }
}

impl SubAssign<BoundValueType> for BoundedValue {
    fn sub_assign(&mut self, rhs: BoundValueType) {
        self.sub_val(rhs);
    }
}

impl PartialEq for BoundedValue {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl PartialEq<BoundValueType> for BoundedValue {
    fn eq(&self, other: &BoundValueType) -> bool {
        self.val == *other
    }
}

impl Eq for BoundedValue {}

impl PartialOrd for BoundedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
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

impl Ord for BoundedValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
