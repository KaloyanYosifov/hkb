/// Min heap
/// 1. Always the most minimum value is on top
/// 2. We have a trait which returns an integer value
/// 3. The trait will be dynamic and boxed
///
/// Implementation
/// 1. Insert:
///     - Put the value in the end
///     - Bubble up
/// 2. Pop
///     - Keep the value we are going to return
///     - Swap with the bottom most element
///     - remove it from the last element
///     - Start from root and heapify to fix the top minimum value

// TODO:
// Maybe we can do traits support here and a user can handle any internal obj they want to keep
// Something like MinHeapValue trait and the user can implement a struct with this trait and the
// struct can have { node: Some(node), node_val: i64, somehting: None }

pub trait Constraints: PartialEq + Eq + PartialOrd + Ord {}

impl<T: PartialEq + Eq + PartialOrd + Ord> Constraints for T {}

#[derive(Debug)]
pub struct MinHeap<T: Constraints> {
    values: Vec<T>,
}

impl<T: Constraints> MinHeap<T> {
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }
}

impl<T: Constraints> MinHeap<T> {
    pub fn insert(&mut self, value: T) {
        self.values.push(value);

        self.heapify_bottom_up();
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.values.len() == 0 {
            None
        } else if self.values.len() == 1 {
            self.values.pop()
        } else {
            self.swap(0, self.values.len() - 1);

            let value = self.values.pop();

            self.heapify_top_down();

            value
        }
    }

    pub fn has_element(&self) -> bool {
        self.values.len() > 0
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    fn heapify_bottom_up(&mut self) {
        let mut index = self.values.len() - 1;

        while index > 0 {
            let parent = (index - 1) / 2;

            if self.values[parent] > self.values[index] {
                self.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    fn heapify_top_down(&mut self) {
        let mut index = 0;

        while index < self.values.len() {
            let left = (index * 2) + 1;
            let right = (index * 2) + 2;
            let mut smallest = index;

            if left < self.values.len() && self.values[left] < self.values[smallest] {
                smallest = left;
            }

            if right < self.values.len() && self.values[right] < self.values[smallest] {
                smallest = right;
            }

            if smallest != index {
                self.swap(smallest, index);
                index = smallest;
            } else {
                break;
            }
        }
    }

    fn swap(&mut self, index1: usize, index2: usize) {
        self.values.swap(index1, index2);
    }
}

#[cfg(test)]
mod tests {
    use super::MinHeap;
    use proptest::prelude::*;

    // TODO: add property testing

    fn create_min_heap() -> MinHeap<i64> {
        let mut min_heap = MinHeap::new();
        let mut values: [usize; 10] = core::array::from_fn(|i| i + 1);

        values.reverse();

        for i in values {
            min_heap.insert(i as i64);
        }

        min_heap
    }

    #[test]
    fn it_can_insert_a_value() {
        let mut min_heap = MinHeap::new();

        min_heap.insert(5);

        assert_eq!(5, min_heap.values[0]);
    }

    #[test]
    fn it_does_not_put_new_high_value_to_the_top() {
        let mut min_heap = create_min_heap();

        min_heap.insert(300);

        assert_eq!(1, min_heap.values[0]);
    }

    #[test]
    fn it_can_get_top_value_from_min_heap() {
        let mut min_heap = create_min_heap();

        let val = min_heap.pop();

        assert_eq!(1, val.unwrap());
        assert_eq!(2, min_heap.values[0]);
    }

    proptest! {
        #[test]
        fn it_puts_the_min_value_at_top(ref values in prop::collection::vec(any::<i64>(), 0..=50)) {
            let mut min_heap = MinHeap::new();

            // Insert all values into the min heap
            for &val in values {
                min_heap.insert(val);
            }

            if let Some(&min_val) = min_heap.values.get(0) {
                assert_eq!(min_val, *values.iter().min().unwrap());
            } else {
                assert!(values.is_empty());
            }
        }

        #[test]
        fn it_puts_the_min_value_at_the_top_after_delete(ref mut values in prop::collection::vec(any::<i64>(), 0..=50)) {
            let mut min_heap = MinHeap::new();

            // Insert all values into the min heap
            for &val in values.iter() {
                min_heap.insert(val);
            }

            values.sort();

            let mut index = 0;
            while min_heap.has_element() {
                let val = min_heap.pop().unwrap();

                assert_eq!(values[index], val);
                index += 1;
            }

            assert!(matches!(min_heap.pop(), None));
        }
    }
}
