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

pub struct MinHeap {
    values: Vec<i64>,
}

impl MinHeap {
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }
}

impl MinHeap {
    pub fn insert(&mut self, value: i64) {
        self.values.push(value);

        self.heapify_bottom_up();
    }

    pub fn pop(&mut self) -> Option<i64> {
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

    fn heapify_bottom_up(&mut self) {
        let mut index = self.values.len() - 1;
        let mut parent = index / 2;

        while self.values[parent] > self.values[index] {
            self.swap(index, parent);
            index = parent;
            parent = index / 2;
        }
    }

    fn heapify_top_down(&mut self) {
        let mut index = 0;

        loop {
            let current_val = self.values[index];
            let left = (index * 2) + 1;
            let right = (index * 2) + 2;

            if left < self.values.len() && right < self.values.len() {
                let left_val = self.values[left];
                let right_val = self.values[right];

                if left_val < right_val && left_val < current_val {
                    self.swap(left, index);
                    index = left;
                } else if right_val < left_val && right_val < current_val {
                    self.swap(right, index);
                    index = right;
                } else {
                    break;
                }
            } else if left < self.values.len() {
                let left_val = self.values[left];

                if left_val < current_val {
                    self.swap(left, index);
                    index = left;
                } else {
                    break;
                }
            } else if right < self.values.len() {
                let right_val = self.values[right];

                if right_val < current_val {
                    self.swap(right, index);
                    index = right;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn swap(&mut self, index1: usize, index2: usize) {
        let temp = self.values[index1];
        self.values[index1] = self.values[index2];
        self.values[index2] = temp;
    }
}

#[cfg(test)]
mod tests {
    use super::MinHeap;

    // TODO: add property testing

    fn create_min_heap() -> MinHeap {
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
    fn it_puts_the_min_value_at_top() {
        let mut min_heap = MinHeap::new();
        let mut values: [usize; 10] = core::array::from_fn(|i| i + 1);

        values.reverse();

        for i in values {
            min_heap.insert(i as i64);
            assert_eq!(i as i64, min_heap.values[0]);
        }

        assert_eq!(1, min_heap.values[0]);
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
}
