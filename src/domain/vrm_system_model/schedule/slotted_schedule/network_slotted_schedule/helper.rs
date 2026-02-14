use std::collections::HashMap;

use crate::domain::vrm_system_model::{
    reservation::{
        probe_reservations::ProbeReservations,
        reservation::{Reservation, ReservationState, ReservationTrait},
        reservation_store::ReservationId,
    },
    schedule::slotted_schedule::network_slotted_schedule::NetworkSlottedSchedule,
};

impl NetworkSlottedSchedule {
    /// Calculates the maximum assignable capacity for a reservation within a specific network time slot.
    ///  
    /// ### Algorithm Logic
    /// 1. Identifies the **source** and **target** nodes for the given `reservation_id`.
    /// 2. Retrieves pre-calculated paths from the **Topology Path Cache**.
    /// 3. For each path, it performs a "bottleneck analysis" preformed
    /// 4. Returns the full requested capacity if at least one path can satisfy it entirely.
    ///    Otherwise, returns the maximum available partial capacity found across all evaluated paths.
    ///
    /// ### Parameters
    /// * `slot_index`: The Requested slot of the SlottedScheduleContext.
    /// * `reservation_id`: Unique identifier for the Reservation.
    ///
    /// ### Returns
    /// * An `i64` representing the **maximum assignable capacity**.
    /// * `i64` - The maximum assignable capacity. Returns `0` if no connectivity exists or all paths are saturated.
    pub fn adjust_requirement_to_slot_capacity(&self, slot_index: i64, reservation_id: ReservationId) -> i64 {
        let start = self.reservation_store.get_start_point(reservation_id);
        let end = self.reservation_store.get_end_point(reservation_id);

        let available_paths = if let (Some(source), Some(target)) = (start, end) {
            self.topology.path_cache.get(&(source, target)).unwrap()
        } else {
            // No Path between source and target found
            return 0;
        };

        let mut available_capacity = 0;

        // Check if all links can handle the requested capacity

        // Iterate through the K-Shortest Paths
        for path in available_paths {
            // Init with capacity of first link
            let path_first_link_id = path.network_links.first().unwrap();

            let mut path_available_capacity = self.resource_store.with_mut_schedule(*path_first_link_id, |schedule| {
                schedule.adjust_requirement_to_slot_capacity(slot_index, self.get_capacity(), reservation_id)
            });

            // Check if all links can handle the requested capacity
            for link_id in &path.network_links {
                path_available_capacity = self.resource_store.with_mut_schedule(*link_id, |schedule| {
                    schedule.adjust_requirement_to_slot_capacity(slot_index, path_available_capacity, reservation_id)
                });

                if path_available_capacity == 0 {
                    break;
                }

                if path_available_capacity < 0 {
                    log::error!("path_available_capacity is below zero should never happen.")
                }
            }
            // Path has enough for the whole capacity
            if path_available_capacity == self.get_capacity() {
                return self.get_capacity();
            } else if path_available_capacity > available_capacity {
                available_capacity = path_available_capacity
            }
        }

        return available_capacity;
    }

    /// Commits a reservation's capacity to a specific network time slot across an available path.
    ///
    /// This method identifies a feasible route from the **K-Shortest Paths** and performs an
    /// update on all links of the chosen path. It ensures that the required
    /// capacity is physically reserved in the `SlottedScheduleContext` of each link.
    ///
    /// ### Parameters
    /// * `slot_index`: The Requested slot of the SlottedScheduleContext.
    /// * `reservation_id`: Unique identifier for the Reservation.
    pub fn insert_reservation_into_slot(&mut self, reservation_id: ReservationId, slot_index: i64) {
        let start = self.reservation_store.get_start_point(reservation_id);
        let end = self.reservation_store.get_end_point(reservation_id);

        let k_shortest_paths = if let (Some(source), Some(target)) = (start.clone(), end.clone()) {
            self.topology.path_cache.get(&(source, target)).unwrap()
        } else {
            // No Path between source and target found
            log::debug!(
                "NetworkPolicyInsertReservationInSlot: Inserting Reservation {:?} into slot {} failed by NetworkPolicy. Because there was no valid path between Source {:?} and Target {:?} found.",
                self.reservation_store.get_name_for_key(reservation_id),
                slot_index,
                start,
                end
            );
            return;
        };

        for path in k_shortest_paths {
            // First test if there is a path free
            let mut free = true;
            for link_id in &path.network_links {
                let link_reserved_capacity = self.reservation_store.get_reserved_capacity(reservation_id);

                let path_available_capacity = self.resource_store.with_mut_schedule(*link_id, |schedule| {
                    schedule.adjust_requirement_to_slot_capacity(slot_index, link_reserved_capacity, reservation_id)
                });

                if path_available_capacity != link_reserved_capacity {
                    free = false;
                    break;
                }
            }

            if free {
                // Found path -> register reservation
                for link_id in &path.network_links {
                    let link_reserved_capacity = self.reservation_store.get_reserved_capacity(reservation_id);

                    self.resource_store.with_mut_schedule(*link_id, |schedule| {
                        schedule.ctx.insert_reservation_into_slot(&reservation_id, link_reserved_capacity, slot_index)
                    });
                }

                // Remember path for reservation and slot
                self.reserved_paths
                    .entry(reservation_id)
                    .or_insert_with(|| {
                        log::debug!(
                            "NetworkSchedule add new Reservation/Slot/Path object for {:?}",
                            self.reservation_store.get_name_for_key(reservation_id)
                        );

                        HashMap::new()
                    })
                    .insert(slot_index, path.clone());

                // Book reserved capacity of Reservation in Slot for Link
                self.ctx.insert_reservation_into_slot(&reservation_id, self.reservation_store.get_reserved_capacity(reservation_id), slot_index);
                return;
            }
        }

        log::error!(
            "NetworkSlottedScheduleInsertReservationFailed: Insert Reservation {:?} failed, because committed reservation has no available path in slot index {}.",
            self.reservation_store.get_name_for_key(reservation_id),
            slot_index
        );
    }

    /// Deletes the reserved capacity of the booked path form all affected Links.
    /// Returns true, if the deletion clean up process was a success otherwise return false.
    pub fn on_delete_reservation(&mut self, reservation_id: ReservationId) -> bool {
        let path_per_slot = if let Some(value) = self.reserved_paths.remove(&reservation_id) {
            value
        } else {
            log::error!(
                "NetworkScheduleDeleteReservationFailed: Deletion of booked path of Reservation {:?} failed.",
                self.reservation_store.get_name_for_key(reservation_id)
            );

            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            return false;
        };

        // For each time slot resolve the booked path
        for (slot_index, path) in path_per_slot {
            for link_id in &path.network_links {
                if !self.resource_store.with_mut_schedule(*link_id, |schedule| {
                    schedule.ctx.delete_reservation_in_slot(reservation_id, self.reservation_store.get_reserved_capacity(reservation_id), slot_index)
                }) {
                    log::error!(
                        "NetworkPolicyDeletionOfReservationFailed: The network path deletion of Reservation {:?} failed. slot_index: {}, path: {:?} the link_id which failed of the processed path {:?}. This link should be empty but part of it is still occupied.",
                        self.reservation_store.get_name_for_key(reservation_id),
                        slot_index,
                        path,
                        link_id
                    );
                    return false;
                }
            }
        }
        return true;
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
            && self.get_capacity() > 0
            && self.get_capacity() < self.ctx.active_reservations.get_reserved_capacity(&id)
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
            let available_capacity: i64 = self.adjust_requirement_to_slot_capacity(current_slot_index, candidate_id);

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

    /// Return the overall minimum cut of the Network
    pub fn get_capacity(&self) -> i64 {
        self.topology.max_bandwidth_all_paths
    }
}
