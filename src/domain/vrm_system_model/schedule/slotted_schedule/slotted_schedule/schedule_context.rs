use crate::domain::vrm_system_model::reservation::probe_reservations::ProbeReservations;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule::slot::Slot;
use crate::domain::vrm_system_model::utils::id::SlottedScheduleId;
use crate::domain::vrm_system_model::utils::load_buffer::{GlobalLoadContext, LoadBuffer};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::i64;
use std::sync::Arc;

const FRAGMENTATION_POWER: f64 = 2.0;

#[derive(Debug, Clone)]
pub struct SlottedScheduleContext {
    /// **Unique identifier** for this SlottedSchedule.
    pub id: SlottedScheduleId,

    /// A list of all time **Slots** defined for this schedule.
    pub slots: Vec<Slot>,

    /// The duration of a single time slot.
    pub slot_width: i64,

    /// The index of the earliest possible slot that can be used for scheduling.
    pub start_slot_index: i64,

    /// The index of the latest possible slot that defines the scheduling window's end.
    pub end_slot_index: i64,

    /// The **absolute start time** (e.g., Unix timestamp) of the current scheduling window being viewed.
    pub scheduling_window_start_time: i64,

    /// The **absolute end time** (e.g., Unix timestamp) of the current scheduling window being viewed.
    pub scheduling_window_end_time: i64,

    /// Internal buffer used to track transient or potential resource load.
    pub load_buffer: LoadBuffer,

    /// A map of all currently **active reservations** associated with this schedule.
    pub active_reservations: Reservations,

    /// Flag indicating if the stored **fragmentation_cache** value is valid and up-to-date.
    pub is_frag_cache_up_to_date: bool,

    /// The cached value of the system **fragmentation**.
    pub fragmentation_cache: f64,

    /// A configuration flag to determine if the system should utilize the **quadratic mean**
    /// or the standard formula for fragmentation calculation.
    pub use_quadratic_mean_fragmentation: bool,

    /// A flag indicating whether fragmentation calculation is required for the **prob requests**.
    pub is_frag_needed: bool,
}

impl SlottedScheduleContext {
    pub fn new(
        id: SlottedScheduleId,
        current_time: i64,
        number_of_real_slots: i64,
        slot_width: i64,
        capacity: i64,
        use_quadratic_mean_fragmentation: bool,
        reservation_store: ReservationStore,
    ) -> Self {
        let mut slots: Vec<Slot> = Vec::new();

        // number_of_real_slots is the number of slots in the considered scheduling window
        for _ in 0..number_of_real_slots {
            slots.push(Slot::new(capacity));
        }

        let mut slotted_context = SlottedScheduleContext {
            id: SlottedScheduleId::new(id),
            slots: slots,
            slot_width: slot_width,
            start_slot_index: 0,
            end_slot_index: -1,
            scheduling_window_start_time: 0,
            scheduling_window_end_time: -1,
            load_buffer: LoadBuffer::new(Arc::new(GlobalLoadContext::new())),
            active_reservations: Reservations::new_empty(reservation_store.clone()),
            is_frag_cache_up_to_date: true,
            fragmentation_cache: 0.0,
            use_quadratic_mean_fragmentation: use_quadratic_mean_fragmentation,
            // TODO Always false
            is_frag_needed: false,
        };

        slotted_context.update(current_time);

        return slotted_context;
    }

    pub fn clear(&mut self) {
        log::warn!("In SlottedSchedule id: {}, where all Slots cleared.", self.id);

        for slot in self.slots.iter_mut() {
            slot.reset();
        }

        self.active_reservations.clear();
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

    /// Computes the **absolute start time** in seconds of a virtual slot.
    pub fn get_slot_start_time(&self, index: i64) -> i64 {
        return index * self.slot_width;
    }

    /// Computes the **absolute end time** in seconds of a virtual slot.
    pub fn get_slot_end_time(&self, index: i64) -> i64 {
        return index * self.slot_width + self.slot_width - 1;
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
    /// **Updates the scheduling window** by advancing the internal time pointers based on the current simulation time.
    ///
    /// This process deletes all reservations that have expired (assigned end time is past the new start time)
    /// and moves the load from the now-expired slots into the `load_buffer` for historical tracking.
    /// Note: Utilized by the SlottedSchedule and NetworkSlottedSchedule
    pub fn update(&mut self, current_time: i64) {
        let new_start_slot_index = self.get_slot_index(current_time);

        if self.start_slot_index < new_start_slot_index {
            self.is_frag_cache_up_to_date = false;
        }

        // key are used to: remove reservation which end earlier than the new start time
        let mut ids_to_remove: HashSet<ReservationId> = HashSet::new();

        for clean_index in self.start_slot_index..new_start_slot_index {
            if let Some(slot) = self.get_slot(clean_index) {
                for id in &slot.reservation_ids {
                    let last_slot_of_reservation = self.get_slot_index(self.active_reservations.get_assigned_end(id));
                    if last_slot_of_reservation == clean_index {
                        ids_to_remove.insert(id.clone());
                    }
                }
            }
        }

        for key in ids_to_remove {
            self.active_reservations.delete_reservation(&key);
        }

        for clean_index in self.start_slot_index..new_start_slot_index {
            let load = if let Some(slot) = self.get_slot(clean_index) { slot.load } else { 0 };
            self.load_buffer.add(load, clean_index);

            if let Some(slot) = self.get_mut_slot(clean_index) {
                slot.reset();
            } else {
                log::error!(
                    "In SlottedSchedule: {} Happened an error during the update process. Slots are now invalid due to fail reset of slot {}.",
                    self.id,
                    clean_index
                )
            }
        }

        // set new Pointer to start and end of the new scheduling window
        self.start_slot_index = new_start_slot_index;
        self.end_slot_index = new_start_slot_index + (self.slots.len() as i64) - 1;

        // set corresponding time borders for the scheduling window
        self.scheduling_window_start_time = self.get_slot_start_time(self.start_slot_index);
        self.scheduling_window_end_time = self.get_slot_end_time(self.end_slot_index);
    }

    /// Validates, if deletion of reservation is possible, sets reservation in state `ReservationState::Rejected` if
    /// Reservation was not reserved before deletion request
    /// Returns true, if deletion process an proceed otherwise false is returned
    pub fn is_reservation_valid_for_deletion(&mut self, id: ReservationId) -> bool {
        // Can not Del unreserved reservation
        if !self.active_reservations.contains_key(&id) {
            log::error!("DEL Reservation form Schedule: {}, However Schedule does not contain reservation with id: {:?}", self.id, id);

            self.active_reservations.set_state(&id, ReservationState::Rejected);
            return false;
        }
        return true;
    }

    /// Deletes the provided ReservationId form the specified slot.
    pub fn delete_reservation_in_slot(&mut self, reservation_id: ReservationId, reservation_reserved_capacity: i64, slot_index: i64) -> bool {
        let slot = self.get_mut_slot(slot_index).expect("Slot was not found.");
        return slot.delete_reservation(reservation_id, reservation_reserved_capacity);
    }

    /// Performs the actual deletion of the reservation in the SlottedScheduleContext
    pub fn delete_reservation(&mut self, id: ReservationId, current_time: i64) {
        // Can not delete already finished reservations
        let task_finished: bool = self.active_reservations.get_assigned_end(&id) <= current_time;

        if task_finished {
            log::error!("Can't deleted reservation {:?} form Schedule: {}, because reservation is already finished.", id, self.id,);
            return;
        }

        let del_res_assigned_start = self.active_reservations.get_assigned_start(&id);
        let del_res_assigned_end = self.active_reservations.get_assigned_end(&id);
        let del_res_reserved_capacity = self.active_reservations.get_reserved_capacity(&id);

        // Delete reservation from schedule
        if !self.active_reservations.delete_reservation(&id) {
            log::error!("Del reservation (id: {:?}) was not possible.", id);
            return;
        }

        // Delete reservation from all occupied slots
        let mut reservation_start_slot_index: i64 = self.get_slot_index(del_res_assigned_start);
        let reservation_end_slot_index: i64 = self.get_slot_index(del_res_assigned_end);

        // Delete only parts that are in the scheduling window
        if reservation_start_slot_index < self.start_slot_index {
            reservation_start_slot_index = self.start_slot_index;
        }

        let slotted_schedule_id = self.id.clone();
        for slot_index in reservation_start_slot_index..=reservation_end_slot_index {
            let slot = self
                .get_mut_slot(slot_index)
                .expect(&format!("In the SlottedSchedule id: {} was the slot with index: {} not found.", slotted_schedule_id, slot_index));

            slot.delete_reservation(id.clone(), del_res_reserved_capacity);
        }

        self.is_frag_cache_up_to_date = false;
        return;
    }

    /// Checks if a given point in time falls within the schedule's defined **scheduling window**.
    pub fn is_time_in_scheduling_window(&self, time: i64) -> bool {
        if time > self.scheduling_window_end_time || time < self.scheduling_window_start_time {
            return false;
        }

        return true;
    }

    /// Returns the best found ReservationId of a probe request
    pub fn get_best_probe_reservation(
        &self,
        probe_reservations: &mut ProbeReservations,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
    ) -> Option<ReservationId> {
        if probe_reservations.is_empty() {
            return None;
        }

        let mut best_candidate = probe_reservations.get_res_id_with_first_start_slot(request_id).expect("Error getting random reservation.").clone();

        for candidate_id in probe_reservations.get_ids() {
            if comparator(best_candidate.clone(), candidate_id) == Ordering::Greater {
                best_candidate = candidate_id.clone();
            }
        }

        probe_reservations.reject_all_probe_reservations_except(best_candidate);
        return Some(best_candidate);
    }

    /// Inserts a new reservation requirement into the specified slot.
    pub fn insert_reservation_into_slot(&mut self, key: &ReservationId, requirement: i64, slot_index: i64) {
        let slot = self.get_mut_slot(slot_index).expect("Slot was not found.");
        slot.insert_reservation(requirement, key.clone());
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
}
