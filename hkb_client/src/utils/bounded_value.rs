use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
};

pub type BoundValueType = usize;

#[derive(Clone)]
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
        let mut val = self.val.checked_sub(val).unwrap_or(self.min_val);

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

    #[allow(dead_code)]
    pub fn set_to_max(&mut self) {
        self.val = self.max_val;
    }

    pub fn get_val(&self) -> BoundValueType {
        self.val
    }

    #[allow(dead_code)]
    pub fn get_max_val(&self) -> BoundValueType {
        self.max_val
    }
}

impl Add<BoundedValue> for BoundedValue {
    type Output = Self;

    fn add(self, rhs: BoundedValue) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.add_val(rhs.get_val());

        new_instance
    }
}

impl Add<BoundValueType> for BoundedValue {
    type Output = Self;

    fn add(self, rhs: BoundValueType) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.add_val(rhs);

        new_instance
    }
}

impl Add<u16> for BoundedValue {
    type Output = Self;

    fn add(self, rhs: u16) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.add_val(rhs as BoundValueType);

        new_instance
    }
}

impl AddAssign<BoundValueType> for BoundedValue {
    fn add_assign(&mut self, rhs: BoundValueType) {
        self.add_val(rhs);
    }
}

impl Sub<BoundedValue> for BoundedValue {
    type Output = Self;

    fn sub(self, rhs: BoundedValue) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.sub_val(rhs.get_val());

        new_instance
    }
}

impl Sub<BoundValueType> for BoundedValue {
    type Output = Self;

    fn sub(self, rhs: BoundValueType) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.sub_val(rhs);

        new_instance
    }
}

impl Sub<u16> for BoundedValue {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self::Output {
        let mut new_instance = self.clone();

        new_instance.sub_val(rhs as BoundValueType);

        new_instance
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

impl Ord for BoundedValue {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.val > other.val {
            std::cmp::Ordering::Greater
        } else if self.val < other.val {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

impl PartialOrd for BoundedValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<BoundValueType> for BoundedValue {
    fn partial_cmp(&self, other: &BoundValueType) -> Option<Ordering> {
        let order = {
            if &self.val > other {
                std::cmp::Ordering::Greater
            } else if &self.val < other {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        };

        Some(order)
    }
}
