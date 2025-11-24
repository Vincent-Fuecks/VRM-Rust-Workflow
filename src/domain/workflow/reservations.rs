use crate::domain::workflow::reservation::{Reservation, ReservationKey};
use std::collections::HashMap;

/// TODO Add Comment
pub struct Reservations {
    pub reservations: HashMap<ReservationKey, Box<dyn Reservation>>,
}

impl Reservations {
    pub fn new(reservation: Reservation) -> Self {
        let key: ReservationKey = ReservationKey {
            id: reservation.get_id(),
        };

        let mut map = HashMap::new();
        map.insert(key, reservation);

        Reservations { reservations: map }
    }

    pub fn new_empty() -> Self {
        Reservations {
            reservations: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.reservations.clear();
    }
}
