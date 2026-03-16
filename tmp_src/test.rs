// Simple Rust code to test Phase 6 migration
pub struct Calculator {
    pub value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn add(&mut self, n: i32) {
        self.value += n;
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}
