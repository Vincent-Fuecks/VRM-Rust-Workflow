use crate::domain::vrm_system_model::{
    reservation::{
        probe_reservations::ProbeReservations,
        reservation::{Reservation, ReservationState, ReservationTrait},
        reservation_store::ReservationId,
    },
    schedule::slotted_schedule::slotted_schedule::SlottedSchedule,
};

impl SlottedSchedule {
    /// Searches for all possible time slots in the schedule where the given reservation request can be fully satisfied.
    ///
    /// This method performs the core **scheduling probe** for resource availability. It iterates through
    /// possible start times within the request's booking interval, clips the search to the scheduling window,
    /// and check for feasibility.
    ///
    /// # Returns
    /// Returns a `Reservations` object containing a map of all feasible reservations (candidates) found.
    /// Each candidate represents a valid assignment time within the schedule's constraints.
    pub fn calculate_schedule(&mut self, id: ReservationId) -> ProbeReservations {
        let mut request_start_boundary: i64 = self.ctx.active_reservations.get_booking_interval_start(&id);
        let mut request_end_boundary: i64 = self.ctx.active_reservations.get_booking_interval_end(&id);
        let initial_duration: i64 = self.ctx.active_reservations.get_task_duration(&id);

        if request_start_boundary == i64::MIN {
            request_start_boundary = 0;
        }

        if request_end_boundary == i64::MIN {
            request_end_boundary = i64::MAX;
        }

        let mut search_results = ProbeReservations::new(id, self.reservation_store.clone());

        if !self.ctx.active_reservations.get_is_moldable(&id)
            && self.capacity > 0
            && self.capacity < self.ctx.active_reservations.get_reserved_capacity(&id)
        {
            return search_results;
        }

        let mut earliest_start_index: i64 = self.ctx.get_slot_index(request_start_boundary);
        earliest_start_index = self.ctx.get_effective_slot_index(earliest_start_index);

        let mut latest_start_index: i64 = self.ctx.get_slot_index(request_end_boundary - initial_duration);
        latest_start_index = self.ctx.get_effective_slot_index(latest_start_index);

        for slot_start_index in earliest_start_index..=latest_start_index {
            if let Some(res_candidate) = self.try_fit_reservation(id, slot_start_index, request_end_boundary) {
                search_results.add_only_reservation(res_candidate);
            }
        }
        return search_results;
    }

    // TODO False implementation should not update the self.active_reservations
    fn try_fit_reservation(&mut self, candidate_id: ReservationId, slot_start_index: i64, request_end_boundary: i64) -> Option<Reservation> {
        // TODO Should be not need, because res is a clone and unlike in the java implementation not the same object.
        // candidate.adjust_capacity(candidate.get_reserved_capacity());

        let mut current_required_capacity = self.ctx.active_reservations.get_reserved_capacity(&candidate_id);

        let mut current_duration: i64 = self.ctx.active_reservations.get_task_duration(&candidate_id);
        let mut start_time = self.ctx.get_slot_start_time(slot_start_index);

        self.ctx.active_reservations.get_booking_interval_start(&candidate_id);

        if start_time < self.ctx.active_reservations.get_booking_interval_start(&candidate_id) {
            start_time = self.ctx.active_reservations.get_booking_interval_start(&candidate_id);
        }

        let mut end_time = start_time + current_duration;
        let mut current_end_slot_index = self.ctx.get_slot_index(end_time);
        let mut is_feasible: bool = true;
        let mut current_slot_index: i64 = slot_start_index;

        while current_slot_index <= current_end_slot_index {
            let available_capacity: i64 = self.adjust_requirement_to_slot_capacity(current_slot_index, current_required_capacity, candidate_id);

            if available_capacity == 0 && current_required_capacity != 0 {
                is_feasible = false;
                break;
            }

            if !self.ctx.active_reservations.get_is_moldable(&candidate_id) && available_capacity != current_required_capacity {
                is_feasible = false;
                break;
            }

            if available_capacity < current_required_capacity {
                self.ctx.active_reservations.adjust_capacity(&candidate_id, available_capacity);
                current_required_capacity = available_capacity;
                current_duration = self.ctx.active_reservations.get_task_duration(&candidate_id);

                end_time = start_time + current_duration;

                if false == self.ctx.is_time_in_scheduling_window(end_time) || end_time > request_end_boundary {
                    is_feasible = false;
                    break;
                }

                current_end_slot_index = self.ctx.get_slot_index(end_time);
            }

            current_slot_index += 1;
        }

        if is_feasible {
            let mut res_candidate_clone = self.ctx.active_reservations.get_reservation_snapshot(&candidate_id);

            res_candidate_clone.set_booking_interval_start(start_time);
            res_candidate_clone.set_booking_interval_end(end_time);
            res_candidate_clone.set_assigned_start(start_time);
            res_candidate_clone.set_assigned_end(end_time);
            res_candidate_clone.set_state(ReservationState::ProbeAnswer);
            return Some(res_candidate_clone);
        }

        return None;
    }

    /// Adjusts the requested resource requirement (**capacity**) to ensure it does not exceed the
    /// **remaining available capacity** in a specific slot.
    /// If the requested capacity is too high, the maximum available capacity for that slot is returned.
    pub fn adjust_requirement_to_slot_capacity(&self, slot_index: i64, capacity: i64, id: ReservationId) -> i64 {
        if let Some(slot) = self.ctx.get_slot(slot_index) {
            return slot.get_adjust_requirement(capacity);
        } else {
            log::error!(
                "SlottedSchedule: {}: requested slot outside of scheduling window. Slot index: {}, window start: {}  window width: {} ReservationId: {:?}",
                self.ctx.id,
                slot_index,
                self.ctx.start_slot_index,
                self.ctx.slots.len() as i64,
                id,
            );

            return 0;
        }
    }
}
