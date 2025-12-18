use crate::domain::vrm_system_model::reservation::{reservation::ReservationState, reservation_store::ReservationId, reservations::Reservations};
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::utils::load_buffer::{LoadMetric, SLOTS_TO_DROP_ON_END, SLOTS_TO_DROP_ON_START};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::i64;
use std::ops::Not;

impl Schedule for super::SlottedSchedule {
    fn clear(&mut self) {
        log::warn!("In SlottedSchedule id: {}, where all Slots cleared.", self.id);

        for slot in self.slots.iter_mut() {
            slot.reset();
        }

        self.active_reservations.clear();
    }

    fn get_simulation_load_metric(&mut self) -> LoadMetric {
        let index_of_first_slot: i64 = self.load_buffer.context.get_first_load() + SLOTS_TO_DROP_ON_START;
        let start_time_of_first_slot: i64 = self.get_slot_start_time(index_of_first_slot);

        let index_of_last_slot: i64 = self.load_buffer.context.get_last_load() - SLOTS_TO_DROP_ON_END;
        let start_time_of_last_slot: i64 = self.get_slot_start_time(index_of_last_slot);

        return self.load_buffer.get_effective_overall_load(self.capacity as f64, start_time_of_first_slot, start_time_of_last_slot);
    }

    fn reserve(&mut self, reservation_id: ReservationId) -> Option<ReservationId> {
        self.update();

        let search_results = self.calculate_schedule(reservation_id);

        match search_results.get_id_with_first_start_slot() {
            Some(reservation_id_with_first_start_slot) => {
                self.is_frag_cache_up_to_date = false;
                self.reserve_without_check(reservation_id_with_first_start_slot);
                return None;
            }

            None => {
                self.active_reservations.set_state(&reservation_id, ReservationState::Rejected);
                return Some(reservation_id);
            }
        }
    }

    fn get_fragmentation(&mut self, frag_start_time: i64, frag_end_time: i64) -> f64 {
        self.update();
        let mut frag_end_time = frag_end_time;

        if frag_end_time == i64::MIN {
            frag_end_time = i64::MAX
        } else if frag_end_time <= frag_start_time {
            log::error!(
                "Request to get fragmentation of Schedule: {}, the fragmentation start time {} was before the fragmentation end time {}.",
                self.id,
                frag_start_time,
                frag_end_time,
            )
        }

        let mut start_slot_index = self.get_slot_index(frag_start_time);
        start_slot_index = self.get_effective_slot_index(start_slot_index);

        let mut end_slot_index = self.get_slot_index(frag_end_time);
        end_slot_index = self.get_effective_slot_index(end_slot_index);

        if self.use_quadratic_mean_fragmentation {
            return self.get_fragmentation_quadratic_mean(start_slot_index, end_slot_index);
        }

        return self.get_fragmentation_resubmit(start_slot_index, end_slot_index);
    }

    fn update(&mut self) {
        let current_time: i64 = self.simulator.get_current_time_in_s();
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

    fn delete_reservation(&mut self, id: ReservationId) {
        // Can not Del unreserved reservation
        if !self.active_reservations.contains_key(&id) {
            log::error!("DEL Reservation form Schedule: {}, However Schedule does not contain reservation with id: {:?}", self.id, id);

            self.active_reservations.set_state(&id, ReservationState::Rejected);
            return;
        }

        // Bring scheduling window up to date
        self.update();

        // Can not delete already finished reservations
        let task_finished: bool = self.active_reservations.get_assigned_end(&id) <= self.simulator.get_current_time_in_s();

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

    fn get_load_metric(&mut self, start_time: i64, mut end_time: i64) -> LoadMetric {
        self.update();

        if end_time == i64::MIN {
            end_time = i64::MAX;
        }

        if end_time < start_time {
            log::error!("Start time must be before end time: SlottedSchedule id: {} is end_time: {} < start_time: {}", self.id, end_time, start_time)
        }

        let mut start_slot_nr = self.get_slot_index(start_time);
        start_slot_nr = self.get_effective_slot_index(start_slot_nr);

        let mut end_slot_nr = self.get_slot_index(end_time);
        end_slot_nr = self.get_effective_slot_index(end_slot_nr);

        let mut reserved_capacity_sum: i64 = 0;

        for real_slot_index in start_slot_nr..=end_slot_nr {
            // TODO int realSlotIndex = this.getRealSlotIndex(startSlotNr); Bug in original VRM  was fixed
            let real_slot_index = self.get_real_slot_index(real_slot_index);
            reserved_capacity_sum += self.get_slot_load(real_slot_index);
        }

        let number_of_slots: i64 = end_slot_nr - start_slot_nr;

        if number_of_slots < 0 {
            log::error!("The number of slots should never be negative.")
        }

        let avg_reserved_capacity: f64 =
            if number_of_slots != 0 { (reserved_capacity_sum as f64) / (number_of_slots as f64) } else { self.capacity as f64 };

        LoadMetric {
            start_time,
            end_time,
            avg_reserved_capacity: avg_reserved_capacity,
            possible_capacity: self.capacity as f64,
            utilization: avg_reserved_capacity / (self.capacity as f64),
        }
    }

    fn get_system_fragmentation(&mut self) -> f64 {
        if self.is_frag_cache_up_to_date.not() {
            self.fragmentation_cache = self.get_fragmentation(self.scheduling_window_start_time, self.scheduling_window_end_time);
            self.is_frag_cache_up_to_date = true;
        }
        return self.fragmentation_cache;
    }

    // TODO Function probe is self.update() in worst case 2N + 1 called --> bottleneck.
    fn probe(&mut self, id: ReservationId) -> Reservations {
        self.update();

        let mut candidates = self.calculate_schedule(id);
        let frag_before: f64 = self.get_system_fragmentation();

        if self.is_frag_needed {
            for candidate_id in candidates.clone().iter() {
                let reserve_answer: Option<ReservationId> = self.reserve(candidate_id.clone());
                let frag_delta: f64 = self.get_system_fragmentation() - frag_before;

                candidates.set_frag_delta(candidate_id, frag_delta);

                match reserve_answer {
                    Some(_) => {}
                    None => {
                        self.delete_reservation(candidate_id.clone());
                    }
                }
            }
        }

        return candidates;
    }

    fn probe_best(
        &mut self,
        request_key: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
    ) -> Option<ReservationId> {
        let possible_reservations: Reservations = self.probe(request_key);
        if possible_reservations.is_empty() {
            return None;
        }

        let mut best_candidate: ReservationId =
            possible_reservations.get_id_with_first_start_slot().expect("Error getting random reservation.").clone();

        for candidate_id in possible_reservations.iter() {
            if comparator(best_candidate.clone(), *candidate_id) == Ordering::Greater {
                best_candidate = candidate_id.clone();
            }
        }

        return Some(best_candidate);
    }

    fn reserve_without_check(&mut self, new_id: ReservationId) {
        for slot_index in self.get_slot_index(self.active_reservations.get_assigned_start(&new_id))
            ..=self.get_slot_index(self.active_reservations.get_assigned_end(&new_id))
        {
            self.insert_reservation_into_slot(&new_id, self.active_reservations.get_reserved_capacity(&new_id), slot_index);
        }

        self.active_reservations.insert(new_id);
        self.active_reservations.set_state(&new_id, ReservationState::ReserveAnswer);
    }

    fn clone_box(&self) -> Box<dyn Schedule> {
        Box::new(self.clone())
    }
}
