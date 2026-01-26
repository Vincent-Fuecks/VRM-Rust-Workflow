use crate::domain::simulator::simulator::{SharedSimulator, SystemSimulator};

use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct MockSimulator {
    pub time: i64,
}

impl MockSimulator {
    pub fn new(time: i64) -> MockSimulator {
        MockSimulator { time }
    }
}

#[derive(Debug, Clone)]
pub struct SharedMockSimulator {
    pub time: Arc<RwLock<i64>>,
}

impl SystemSimulator for SharedMockSimulator {
    fn get_current_time_in_s(&self) -> i64 {
        *self.time.read().unwrap()
    }
    fn clone_box(&self) -> SharedSimulator {
        todo!()
    }

    fn get_current_time_in_ms(&self) -> i64 {
        todo!()
    }
}

impl SystemSimulator for MockSimulator {
    fn get_current_time_in_s(&self) -> i64 {
        self.time
    }
    fn get_current_time_in_ms(&self) -> i64 {
        self.time
    }

    fn clone_box(&self) -> SharedSimulator {
        SharedSimulator(Arc::new(self.clone()))
    }
}
