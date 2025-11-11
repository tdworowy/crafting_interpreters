use crate::grow_capacity;

#[derive(Debug, PartialEq)]
pub struct ValueArray {
    pub count: i32,
    pub capacity: i32,
    pub values: Vec<u64>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            count: 0,
            capacity: 0,
            values: Vec::new(),
        }
    }
    pub fn write(&mut self, value: u64) {
        if self.capacity < self.count + 1 {
            self.capacity = grow_capacity!(self.capacity);
            self.values.resize(self.capacity as usize, 0);
        }
        self.values[self.count as usize] = value;
        self.count += 1;
    }
}

#[test]
fn test_value_array() {
    let mut value_array = ValueArray::new();
    assert_eq!(value_array.count, 0);
    assert_eq!(value_array.capacity, 0);
    assert_eq!(value_array.values, vec![]);

    value_array.write(123);

    assert_eq!(value_array.count, 1);
    assert_eq!(value_array.capacity, 8);
    assert_eq!(value_array.values, vec![123, 0, 0, 0, 0, 0, 0, 0]);
}
