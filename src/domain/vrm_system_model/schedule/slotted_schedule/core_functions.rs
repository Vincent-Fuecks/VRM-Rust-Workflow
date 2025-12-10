use crate::domain::vrm_system_model::reservation::{
    reservation::{Reservation, ReservationKey, ReservationState},
    reservations::Reservations,
};
use crate::domain::vrm_system_model::schedule::slot::Slot;
use crate::domain::vrm_system_model::utils::load_buffer::{SLOTS_TO_DROP_ON_END, SLOTS_TO_DROP_ON_START};

use std::i64;
use std::ops::Not;

impl super::SlottedSchedule {
    /// Function is used for testing.
    pub fn set_slot_load(&mut self, index: usize, load: i64) {
        if index < self.slots.len() {
            self.slots[index].load = load;
        } else {
            panic!("Slot index must < total number of all slots.")
        }
    }

    /// Calculates the **virtual index** of the time slot that contains the given point in time.
    ///
    /// This index is an abstract representation based on the `slot_width` and may exceed the actual number
    /// of allocated slots (real slots) (`self.slots.len()`).
    ///
    /// **Note:** A negative input time will always yield an index of $0$.
    pub fn get_slot_index(&self, time: i64) -> i64 {
        let index: i64 = (time as f64 / self.slot_width as f64).floor() as i64;

        if index < 0 {
            log::error!("The requested slot index is negative ({}), because the requested time was negative: {}", index, time,);

            return 0;
        }

        return index;
    }

    /// Checks if a given point in time falls within the schedule's defined **scheduling window**.
    pub fn is_time_in_scheduling_window(&self, time: i64) -> bool {
        if time > self.scheduling_window_end_time || time < self.scheduling_window_start_time {
            return false;
        }

        return true;
    }

    /// Adjusts the requested resource requirement (**capacity**) to ensure it does not exceed the
    /// **remaining available capacity** in a specific slot.
    /// If the requested capacity is too high, the maximum available capacity for that slot is returned.
    pub fn adjust_requirement_to_slot_capacity(&self, slot_index: i64, capacity: i64, reservation_key: ReservationKey) -> i64 {
        if let Some(slot) = self.get_slot(slot_index) {
            return slot.get_adjust_requirement(capacity);
        } else {
            log::error!(
                "SlottedSchedule: {}: requested slot outside of scheduling window. Slot index: {}, window start: {}  window width: {} reservation: {}",
                self.id,
                slot_index,
                self.start_slot_index,
                self.slots.len() as i64,
                reservation_key,
            );

            return 0;
        }
    }

    /// Computes a  **real index** in `slots` to a corresponding **virtual slot index** in the
    /// overall schedule timeline.
    pub fn get_real_slot_index(&self, index: i64) -> i64 {
        return (index % (self.slots.len() as i64)) as i64;
    }

    /// Retrieves the `Slot` corresponding to the given **virtual index**, if it exists within the current window.
    /// The index is virtual, meaning it represents a point in the schedule's timeline, and must be
    /// mapped to a real index in the internal slot vector via `get_real_slot_index`.
    pub fn get_slot(&self, index: i64) -> Option<&Slot> {
        if index < 0 {
            return None;
        }

        if index < self.start_slot_index || index > (self.end_slot_index + 1) {
            return None;
        }

        let real_index: i64 = self.get_real_slot_index(index);
        return self.slots.get(real_index as usize);
    }

    /// Retrieves the `Slot` corresponding to the given **virtual index**, if it exists within the current window.
    /// The index is virtual, meaning it represents a point in the schedule's timeline, and must be
    /// mapped to a real index in the internal slot vector via `get_real_slot_index`.
    pub fn get_mut_slot(&mut self, index: i64) -> Option<&mut Slot> {
        if index < 0 {
            return None;
        }

        if index < self.start_slot_index || index > (self.end_slot_index + 1) {
            return None;
        }

        let real_index: i64 = self.get_real_slot_index(index);
        return self.slots.get_mut(real_index as usize);
    }

    /// Inserts a new reservation requirement into the specified slot.
    pub fn insert_reservation_into_slot(&mut self, key: &ReservationKey, requirement: i64, slot_index: i64) {
        let slot = self.get_mut_slot(slot_index).expect("Slot was not found.");
        slot.insert_reservation(requirement, key.clone());
    }

    /// Limits a given **virtual slot index** to ensure it is bounded by the current schedule window.
    /// This is used to constrain searches and operations to the slots that are currently managed
    /// and tracked by the schedule, preventing out-of-bounds index access in the virtual timeline.
    pub fn get_effective_slot_index(&self, slot_index: i64) -> i64 {
        let mut effective_slot_index: i64 = slot_index;

        if effective_slot_index < self.start_slot_index {
            effective_slot_index = self.start_slot_index;
        }

        if effective_slot_index > self.end_slot_index {
            effective_slot_index = self.end_slot_index;
        }

        return effective_slot_index;
    }

    /// Computes the **absolute start time** in seconds of a virtual slot.
    pub fn get_slot_start_time(&self, index: i64) -> i64 {
        return index * self.slot_width;
    }

    /// Computes the **effective start time** based on the first load index plus slots to drop.
    pub fn get_effective_start_time(&self) -> i64 {
        let index_of_first_slot = self.load_buffer.context.get_first_load() + SLOTS_TO_DROP_ON_START;
        return self.get_slot_start_time(index_of_first_slot);
    }

    /// Computes the **effective end time** based on the last load index minus slots to drop.
    pub fn get_effective_end_time(&self) -> i64 {
        let index_of_last_slot = self.load_buffer.context.get_last_load() - SLOTS_TO_DROP_ON_END;
        return self.get_slot_start_time(index_of_last_slot);
    }

    /// Retrieves the current resource load (reserved capacity) for a slot at a given index.
    /// **Note:** If the slot is not found, an error is logged, and **0** is returned.
    pub fn get_slot_load(&self, index: i64) -> i64 {
        match self.get_slot(index) {
            Some(slot) => slot.load,
            None => {
                log::error!(
                    "In the SlottedSchedule {} was of a with the index {} the load requested. However with the slot index exists not slot.",
                    self.id,
                    index,
                );
                return 0;
            }
        }
    }

    /// Returns the **actual number of slots** currently instantiated and managed by the schedule
    /// (`self.slots.len()`).
    pub fn get_number_of_real_slots(&self) -> i64 {
        return self.slots.len() as i64;
    }

    /// Computes the **absolute end time** in seconds of a virtual slot.
    pub fn get_slot_end_time(&self, index: i64) -> i64 {
        return index * self.slot_width + self.slot_width - 1;
    }

    /// Searches for all possible time slots in the schedule where the given reservation request can be fully satisfied.
    ///
    /// This method performs the core **scheduling probe** for resource availability. It iterates through
    /// possible start times within the request's booking interval, clips the search to the scheduling window,
    /// and check for feasibility.
    ///
    /// # Returns
    /// Returns a `Reservations` object containing a map of all feasible reservations (candidates) found.
    /// Each candidate represents a valid assignment time within the schedule's constraints.
    pub fn calculate_schedule(&self, key: ReservationKey) -> Reservations {
        let mut request_start_boundary: i64 = self.active_reservations.get_booking_interval_start(&key);
        let mut request_end_boundary: i64 = self.active_reservations.get_booking_interval_end(&key);
        let initial_duration: i64 = self.active_reservations.get_task_duration(&key);

        if request_start_boundary == i64::MIN {
            request_start_boundary = 0;
        }

        if request_end_boundary == i64::MIN {
            request_end_boundary = i64::MAX;
        }

        let mut search_results = Reservations::new_empty();

        if self.active_reservations.get_is_moldable(&key).not()
            && self.capacity > 0
            && self.capacity < self.active_reservations.get_reserved_capacity(&key)
        {
            return search_results;
        }

        let mut earliest_start_index: i64 = self.get_slot_index(request_start_boundary);
        earliest_start_index = self.get_effective_slot_index(earliest_start_index);

        let mut latest_start_index: i64 = self.get_slot_index(request_end_boundary - initial_duration);
        latest_start_index = self.get_effective_slot_index(latest_start_index);

        for slot_start_index in earliest_start_index..=latest_start_index {
            let candidate: Box<dyn Reservation + 'static> = self.active_reservations.box_clone(&key);

            if let Some(candidate) = self.try_fit_reservation(candidate, slot_start_index, request_end_boundary) {
                search_results.insert(candidate.get_id(), candidate);
            }
        }
        return search_results;
    }

    fn try_fit_reservation(
        &self,
        mut candidate: Box<dyn Reservation + 'static>,
        slot_start_index: i64,
        request_end_boundary: i64,
    ) -> Option<Box<dyn Reservation + 'static>> {
        // TODO Should be not need, because res is a clone and unlike in the java implementation not the same object.
        // candidate.adjust_capacity(candidate.get_reserved_capacity());

        let mut current_required_capacity = candidate.get_reserved_capacity();
        let mut current_duration: i64 = candidate.get_task_duration();
        let mut start_time = self.get_slot_start_time(slot_start_index);

        if start_time < candidate.get_booking_interval_start() {
            start_time = candidate.get_booking_interval_start();
        }

        let mut end_time = start_time + current_duration;
        let mut current_end_slot_index = self.get_slot_index(end_time);
        let mut is_feasible: bool = true;
        let mut current_slot_index: i64 = slot_start_index;

        while current_slot_index <= current_end_slot_index {
            let available_capacity: i64 =
                self.adjust_requirement_to_slot_capacity(current_slot_index, current_required_capacity, candidate.get_id().clone());

            if available_capacity == 0 && current_required_capacity != 0 {
                is_feasible = false;
                break;
            }

            if candidate.is_moldable().not() && available_capacity != current_required_capacity {
                is_feasible = false;
                break;
            }

            if available_capacity < current_required_capacity {
                candidate.adjust_capacity(available_capacity);
                current_required_capacity = available_capacity;
                current_duration = candidate.get_task_duration();

                end_time = start_time + current_duration;

                if false == self.is_time_in_scheduling_window(end_time) || end_time > request_end_boundary {
                    is_feasible = false;
                    break;
                }

                current_end_slot_index = self.get_slot_index(end_time);
            }

            current_slot_index += 1;
        }

        if is_feasible {
            candidate.set_booking_interval_start(start_time);
            candidate.set_booking_interval_end(end_time);
            candidate.set_assigned_start(start_time);
            candidate.set_assigned_end(end_time);
            candidate.set_state(ReservationState::ProbeAnswer);
            return Some(candidate);
        }

        return None;
    }
}
