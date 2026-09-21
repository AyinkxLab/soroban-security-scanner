//! A plain Rust library. This fixture must be classified as generic Rust.

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Counter {
    value: i64,
}

impl Counter {
    pub fn new() -> Self {
        Counter { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }
}
