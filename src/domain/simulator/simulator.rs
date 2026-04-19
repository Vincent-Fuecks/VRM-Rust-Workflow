use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalClockDto {
    pub is_simulation: bool,
}

#[derive(Debug)]
pub struct GlobalClock {
    pub is_simulation: bool,
    pub reference_start_time: AtomicI64,
}

impl GlobalClock {
    pub fn new(is_simulation: bool) -> Self {
        let start_at = if is_simulation { 0 } else { SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64 };

        Self { is_simulation: is_simulation, reference_start_time: AtomicI64::new(start_at) }
    }

    pub fn get_system_time_s(&self) -> i64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i64;

        if self.is_simulation {
            return now - self.reference_start_time.load(Ordering::Relaxed);
        }

        return now;
    }
}
