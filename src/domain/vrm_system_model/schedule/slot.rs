use std::collections::HashSet;

use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;

/// TODO Add Comment
#[derive(Debug, Clone)]
pub struct Slot {
    /// The current reserved capacity, which is assigned to this slot by active reservations.
    pub load: i64,

    /// The maximum physical capacity of the resource managed by this slot.
    /// This value remains constant.
    pub capacity: i64,

    /// A set of **unique keys** identifying all reservations currently occupying
    /// capacity within this time slot. Used for quick lookup and deletion.
    pub reservation_keys: HashSet<ReservationKey>,
}

impl Slot {
    pub fn new(capacity: i64) -> Self {
        Slot { capacity: capacity, load: 0, reservation_keys: HashSet::new() }
    }

    /// Checks the available capacity in the slot against a potential reservation requirement.
    /// This function determines the maximum capacity that can be satisfied, up to the
    /// requested `requirements`.
    pub fn get_adjust_requirement(&self, requirements: i64) -> i64 {
        let res_left = self.capacity - self.load;

        if res_left >= requirements {
            return requirements;
        }

        return res_left;
    }

    /// Resets the slot state by clearing all associated reservation keys and setting the
    /// current resource load back to zero.
    pub fn reset(&mut self) {
        self.load = 0;
        self.reservation_keys.clear();
    }

    /// Inserts a new reservation into the slot, updating the current load and tracking of the keys.
    ///
    /// # Returns
    /// `true` if the key was newly inserted and load was adjusted;
    /// `false` if the key was already present or load was to large for slot.
    pub fn insert_reservation(&mut self, requirement: i64, key: ReservationKey) -> bool {
        if self.load + requirement > self.capacity {
            log::error!(
                "New reservation (id: {}) exceeds capacity of slot. Load with request: {} Slot capacity: {}",
                key,
                self.load + requirement,
                self.capacity
            );

            return false;
        }

        if self.reservation_keys.insert(key.clone()) {
            self.load += requirement;
            true
        } else {
            // Log a warning if a duplicate is inserted, as load was not increased
            log::warn!("Attempted to insert duplicate reservation key (id: {:?}). Load was not updated.", key.clone());
            false
        }
    }

    /// Deletes a reservation from the slot, reducing the current load by the reserved capacity.
    ///
    /// # Returns
    /// `true` if the reservation key was found and removed, `false` otherwise (and an error is logged).
    pub fn delete_reservation(&mut self, key: ReservationKey, reservation_reserved_capacity: i64) -> bool {
        if self.load < reservation_reserved_capacity {
            log::error!("Deletion of reservation (id: {}) results in a negative load of slot --> Signals an error in the implementation.", key);
        }

        match self.reservation_keys.remove(&key) {
            true => {
                self.load -= reservation_reserved_capacity;
                true
            }
            false => {
                log::error!("Deletion of reservation (id: {}) was not possible, because reservation with provided id doesn't exist.", key);
                false
            }
        }
    }
}
