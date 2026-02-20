use std::cmp::Ordering;

use crate::domain::vrm_system_model::{
    reservation::{probe_reservations::ProbeReservations, reservation::ReservationState, reservation_store::ReservationId},
    schedule::slotted_schedule::slotted_schedule::SlottedSchedule,
    scheduler_trait::Schedule,
    utils::load_buffer::{LoadMetric, SLOTS_TO_DROP_ON_END, SLOTS_TO_DROP_ON_START},
};

impl Schedule for SlottedSchedule {
    fn clear(&mut self) {
        self.ctx.clear();
    }

    fn clone_box(&self) -> Box<dyn Schedule> {
        Box::new(self.clone())
    }

    fn delete_reservation(&mut self, reservation_id: ReservationId) {
        if self.ctx.is_reservation_valid_for_deletion(reservation_id) {
            // Bring scheduling window up to date
            self.update();
            // Delete Reservation from SlottedSchedule
            self.ctx.delete_reservation(reservation_id, self.simulator.get_current_time_in_s());
        }
    }

    /// TODO Function probe utilizes self.update() in worst case 2N + 1 times --> potential optimization.
    /// Note is the same function as in NetworkSlottedSchedule (please check if this version must be adjusted too)
    fn probe(&mut self, id: ReservationId) -> ProbeReservations {
        self.update();

        let candidates = self.calculate_schedule(id);
        let frag_before: f64 = self.get_system_fragmentation();

        if self.ctx.is_frag_needed {
            for candidate_id in candidates.get_ids() {
                let reserve_answer: Option<ReservationId> = self.reserve(candidate_id);
                let frag_delta: f64 = self.get_system_fragmentation() - frag_before;

                self.reservation_store.set_frag_delta(candidate_id, frag_delta);

                match reserve_answer {
                    Some(reserve_answer) => self.delete_reservation(reserve_answer),
                    None => {
                        panic!("Error in cleaning SlottedSchedule form probe request.")
                    }
                }
            }
        }

        return candidates;
    }

    /// Note is the same function as in NetworkSlottedSchedule (please check if this version must be adjusted too)
    fn probe_best(
        &mut self,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
    ) -> Option<ReservationId> {
        let mut probe_reservations = self.probe(request_id);
        return self.ctx.get_best_probe_reservation(&mut probe_reservations, request_id, comparator);
    }

    /// Note is the same function as in NetworkSlottedSchedule (please check if this version must be adjusted too)
    fn reserve(&mut self, reservation_id: ReservationId) -> Option<ReservationId> {
        self.update();

        let mut probe_reservations = self.calculate_schedule(reservation_id);

        match probe_reservations.get_res_id_with_first_start_slot(reservation_id) {
            Some(res_id) => {
                self.ctx.is_frag_cache_up_to_date = false;
                self.reserve_without_check(res_id);
                probe_reservations.reject_all_probe_reservations_except(res_id);
                return None;
            }
            None => {
                self.ctx.active_reservations.set_state(&reservation_id, ReservationState::Rejected);
                // TODO Del Reservation form Slots ProbeReservation
                probe_reservations.reject_all_probe_reservations();
                return Some(reservation_id);
            }
        }
    }

    /// Note is the same function as in NetworkSlottedSchedule (please check if this version must be adjusted too)
    fn reserve_without_check(&mut self, id: ReservationId) {
        for slot_index in self.ctx.get_slot_index(self.ctx.active_reservations.get_assigned_start(&id))
            ..=self.ctx.get_slot_index(self.ctx.active_reservations.get_assigned_end(&id))
        {
            self.ctx.insert_reservation_into_slot(&id, self.reservation_store.get_reserved_capacity(id), slot_index);
        }

        self.ctx.active_reservations.insert(id);
        self.ctx.active_reservations.set_state(&id, ReservationState::ReserveAnswer);
    }

    fn update(&mut self) {
        self.ctx.update(self.simulator.get_current_time_in_s());
    }

    fn get_fragmentation(&mut self, frag_start_time: i64, frag_end_time: i64) -> f64 {
        self.update();
        let mut frag_end_time = frag_end_time;

        if frag_end_time == i64::MIN {
            frag_end_time = i64::MAX
        } else if frag_end_time <= frag_start_time {
            log::error!(
                "Request to get fragmentation of Schedule: {}, the fragmentation start time {} was before the fragmentation end time {}.",
                self.ctx.id,
                frag_start_time,
                frag_end_time,
            )
        }

        let mut start_slot_index = self.ctx.get_slot_index(frag_start_time);
        start_slot_index = self.ctx.get_effective_slot_index(start_slot_index);

        let mut end_slot_index = self.ctx.get_slot_index(frag_end_time);
        end_slot_index = self.ctx.get_effective_slot_index(end_slot_index);

        if self.ctx.use_quadratic_mean_fragmentation {
            return self.get_fragmentation_quadratic_mean(start_slot_index, end_slot_index);
        }

        return self.get_fragmentation_resubmit(start_slot_index, end_slot_index);
    }

    fn get_system_fragmentation(&mut self) -> f64 {
        if !self.ctx.is_frag_cache_up_to_date {
            self.ctx.fragmentation_cache = self.get_fragmentation(self.ctx.scheduling_window_start_time, self.ctx.scheduling_window_end_time);
            self.ctx.is_frag_cache_up_to_date = true;
        }

        return self.ctx.fragmentation_cache;
    }

    fn get_load_metric(&self, start_time: i64, mut end_time: i64) -> LoadMetric {
        if end_time == i64::MIN {
            end_time = i64::MAX;
        }

        if end_time < start_time {
            log::error!(
                "Start time must be before end time: SlottedSchedule id: {} is end_time: {} < start_time: {}",
                self.ctx.id,
                end_time,
                start_time
            )
        }

        let mut start_slot_nr = self.ctx.get_slot_index(start_time);
        start_slot_nr = self.ctx.get_effective_slot_index(start_slot_nr);

        let mut end_slot_nr = self.ctx.get_slot_index(end_time);
        end_slot_nr = self.ctx.get_effective_slot_index(end_slot_nr);

        let mut reserved_capacity_sum: i64 = 0;

        for real_slot_index in start_slot_nr..=end_slot_nr {
            let real_slot_index = self.ctx.get_real_slot_index(real_slot_index);
            reserved_capacity_sum += self.ctx.get_slot_load(real_slot_index);
        }
        let mut number_of_slots = 0;

        if self.ctx.slots.len() > 0 {
            number_of_slots = end_slot_nr - start_slot_nr + 1;
        }

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

    fn get_load_metric_up_to_date(&mut self, start_time: i64, end_time: i64) -> LoadMetric {
        self.update();
        self.get_load_metric(start_time, end_time)
    }

    fn get_simulation_load_metric(&mut self) -> LoadMetric {
        let index_of_first_slot: i64 = self.ctx.load_buffer.context.get_first_load() + SLOTS_TO_DROP_ON_START;
        let start_time_of_first_slot: i64 = self.ctx.get_slot_start_time(index_of_first_slot);

        let index_of_last_slot: i64 = self.ctx.load_buffer.context.get_last_load() - SLOTS_TO_DROP_ON_END;
        let start_time_of_last_slot: i64 = self.ctx.get_slot_start_time(index_of_last_slot);

        return self.ctx.load_buffer.get_effective_overall_load(self.capacity as f64, start_time_of_first_slot, start_time_of_last_slot);
    }
}
