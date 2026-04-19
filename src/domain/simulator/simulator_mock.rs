use crate::domain::simulator::simulator::{SharedSimulator, SystemSimulator};

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct SharedMockSimulator {}

impl SystemSimulator for SharedMockSimulator {
    fn get_current_time_in_s(&self) -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64
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
