use std::cmp::Ordering;

use crate::domain::vrm_system_model::{
    reservation::{probe_reservations::ProbeReservations, reservation::ReservationState, reservation_store::ReservationId},
    schedule::slotted_schedule::network_slotted_schedule::NetworkSlottedSchedule,
    scheduler_trait::Schedule,
    utils::load_buffer::LoadMetric,
};

impl Schedule for NetworkSlottedSchedule {
    fn clear(&mut self) {
        log::debug!("Clear NetworkSlottedSchedule {}", self.ctx.id);

        for (_, link) in &mut self.topology.network_links {
            link.schedule.clear();
        }

        self.reserved_paths.clear();

        // Clear general SlottedSchedule
        self.ctx.clear();
    }

    fn clone_box(&self) -> Box<dyn Schedule> {
        Box::new(self.clone())
    }

    fn delete_reservation(&mut self, reservation_id: ReservationId) {
        if self.on_delete_reservation(reservation_id) {
            if self.ctx.is_reservation_valid_for_deletion(reservation_id) {
                // Bring scheduling window up to date
                self.update();
                // Delete Reservation from SlottedSchedule
                self.ctx.delete_reservation(reservation_id, self.simulator.get_current_time_in_s());
            }
        }
    }

    /// TODO Function probe utilizes self.update() in worst case 2N + 1 times --> potential optimization.
    /// Note is the same function as in SlottedSchedule (please check if this version must be adjusted too)
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
                    Some(_) => {}
                    None => {
                        self.delete_reservation(candidate_id.clone());
                    }
                }
            }
        }

        return candidates;
    }

    /// Note is the same function as in SlottedSchedule (please check if this version must be adjusted too)
    fn probe_best(
        &mut self,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
    ) -> Option<ReservationId> {
        let mut probe_reservations = self.probe(request_id);
        return self.ctx.get_best_probe_reservation(&mut probe_reservations, request_id, comparator);
    }

    /// Note is the same function as in SlottedSchedule (please check if this version must be adjusted too)
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
                probe_reservations.reject_all_probe_reservations();
                return Some(reservation_id);
            }
        }
    }
    /// Note is the same function as in SlottedSchedule (please check if this version must be adjusted too)
    fn reserve_without_check(&mut self, id: ReservationId) {
        for slot_index in self.ctx.get_slot_index(self.ctx.active_reservations.get_assigned_start(&id))
            ..=self.ctx.get_slot_index(self.ctx.active_reservations.get_assigned_end(&id))
        {
            self.insert_reservation_into_slot(id, slot_index);
        }

        self.ctx.active_reservations.insert(id);
        self.ctx.active_reservations.set_state(&id, ReservationState::ReserveAnswer);
    }

    fn update(&mut self) {
        self.ctx.update(self.simulator.get_current_time_in_s());
    }

    // TODO Not Implemented
    fn get_fragmentation(&mut self, _frag_start_time: i64, _frag_end_time: i64) -> f64 {
        return -1.0;
    }

    // TODO Not Implemented
    fn get_system_fragmentation(&mut self) -> f64 {
        return -1.0;
    }

    // TODO Not Implemented
    fn get_load_metric(&self, _start_time: i64, _end_time: i64) -> LoadMetric {
        LoadMetric::new(-1, -1, -1.0, -1.0, 0.0)
    }

    // TODO Not Implemented
    fn get_load_metric_up_to_date(&mut self, _start_time: i64, _end_time: i64) -> LoadMetric {
        LoadMetric::new(-1, -1, -1.0, -1.0, 0.0)
    }

    // TODO Not Implemented
    fn get_simulation_load_metric(&mut self) -> LoadMetric {
        LoadMetric::new(-1, -1, -1.0, -1.0, 0.0)
    }
}
