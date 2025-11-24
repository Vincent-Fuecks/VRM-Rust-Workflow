use std::collections::HashSet;

use crate::domain::workflow::reservation::ReservationKey;

/// TODO Add Comment
pub struct Slot {
    pub load: i64,
    pub capacity: i64,
    pub reservation_keys: HashSet<ReservationKey>,
}

impl Slot {
    pub fn new(capacity: i64) -> Self {
        Slot {
            capacity: capacity,
            load: 0,
            reservation_keys: HashSet::new(),
        }
    }

    pub fn reset(&mut self) {
        self.load = 0;
        self.reservation_keys.clear()
    }
}
