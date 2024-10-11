use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrdtCounter {
    pub count: i32,
}

impl CrdtCounter {
    pub fn new() -> Self {
        CrdtCounter { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn merge(&mut self, other: &CrdtCounter) {
        self.count += other.count;
    }
}

impl Default for CrdtCounter {
    fn default() -> Self {
        Self::new()
    }
}

// GCounter (Grow-only Counter)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GCounter {
    pub count: i32,
}

impl GCounter {
    pub fn new() -> Self {
        GCounter { count: 0 }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn merge(&mut self, other: &GCounter) {
        self.count = self.count.max(other.count);
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

// PNCounter (Positive-Negative Counter)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PNCounter {
    pub positive: i32,
    pub negative: i32,
}

impl PNCounter {
    pub fn new() -> Self {
        PNCounter {
            positive: 0,
            negative: 0,
        }
    }

    pub fn increment(&mut self) {
        self.positive += 1;
    }

    pub fn decrement(&mut self) {
        self.negative += 1;
    }

    pub fn value(&self) -> i32 {
        self.positive - self.negative
    }

    pub fn merge(&mut self, other: &PNCounter) {
        self.positive = self.positive.max(other.positive);
        self.negative = self.negative.max(other.negative);
    }
}

impl Default for PNCounter {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement other CRDTs
