use crate::domain::vrm_system_model::{
    reservation::{probe_reservations::ProbeReservations, reservation::ReservationState, reservation_store::ReservationId},
    schedule::{
        schedule_trait::Schedule,
        slotted_schedule::{slotted_schedule_context::SlottedScheduleContext, strategy::strategy_trait::SlottedScheduleStrategy},
    },
    utils::load_buffer::LoadMetric,
};

impl<S: SlottedScheduleStrategy> Schedule for SlottedScheduleContext<S> {
    fn clear(&mut self) {
        S::on_clear(self);
        self.slots.clear();
        self.update();
    }

    fn clone_box(&self) -> Box<dyn Schedule> {
        Box::new(self.clone())
    }

    fn get_fragmentation(&mut self, frag_start_time: i64, frag_end_time: i64) -> f64 {
        S::get_fragmentation(self, frag_start_time, frag_end_time)
    }

    fn get_load_metric(&self, start_time: i64, end_time: i64) -> LoadMetric {
        S::get_load_metric(self, start_time, end_time)
    }

    fn get_load_metric_up_to_date(&mut self, start_time: i64, end_time: i64) -> LoadMetric {
        self.update();
        S::get_load_metric(self, start_time, end_time)
    }

    fn get_simulation_load_metric(&mut self) -> LoadMetric {
        S::get_simulation_load_metric(self)
    }

    fn get_system_fragmentation(&mut self) -> f64 {
        S::get_system_fragmentation(self)
    }

    fn probe(&mut self, id: ReservationId) -> ProbeReservations {
        self.update();

        let candidates = self.calculate_schedule(id);
        let frag_before: f64 = self.get_system_fragmentation();

        // TODO
        // if self.is_frag_needed {
        //     for candidate_id in candidates.get_ids() {
        //         let reserve_answer: Option<ReservationId> = self.reserve(candidate_id);
        //         let frag_delta: f64 = self.get_system_fragmentation() - frag_before;

        //         self.reservation_store.set_frag_delta(candidate_id, frag_delta);

        //         match reserve_answer {
        //             Some(reserve_answer) => self.delete_reservation(reserve_answer),
        //             None => {
        //                 panic!("Error in cleaning SlottedSchedule form probe request.")
        //             }
        //         }
        //     }
        // }

        return candidates;
    }

    fn probe_best(
        &mut self,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> std::cmp::Ordering,
    ) -> Option<ReservationId> {
        let mut probe_reservations = self.probe(request_id);
        return self.get_best_probe_reservation(&mut probe_reservations, request_id, comparator);
    }

    fn delete_reservation(&mut self, reservation_id: ReservationId) {
        if self.is_reservation_valid_for_deletion(reservation_id) {
            // Bring scheduling window up to date
            self.update();
            // Delete Reservation from SlottedSchedule
            self.delete_reservation(reservation_id);
        }
    }

    fn reserve(&mut self, reservation_id: ReservationId) -> Option<ReservationId> {
        self.update();

        let mut probe_reservations = self.calculate_schedule(reservation_id);
        return None;
        // TODO
        // match probe_reservations.get_res_id_with_first_start_slot(reservation_id) {
        //     Some(res_id) => {
        //         self.ctx.is_frag_cache_up_to_date = false;
        //         self.reserve_without_check(res_id);
        //         probe_reservations.reject_all_probe_reservations_except(res_id);
        //         return None;
        //     }
        //     None => {
        //         self.active_reservations.set_state(&reservation_id, ReservationState::Rejected);
        //         // TODO Del Reservation form Slots ProbeReservation
        //         probe_reservations.reject_all_probe_reservations();
        //         return Some(reservation_id);
        //     }
        // }
    }

    fn reserve_without_check(&mut self, reservation_id: ReservationId) {
        for slot_index in self.get_slot_index(self.active_reservations.get_assigned_start(&reservation_id))
            ..=self.get_slot_index(self.active_reservations.get_assigned_end(&reservation_id))
        {
            S::insert_reservation_into_slot(self, self.reservation_store.get_reserved_capacity(reservation_id), slot_index, reservation_id);
        }

        self.active_reservations.insert(reservation_id);
        self.active_reservations.set_state(&reservation_id, ReservationState::ReserveAnswer);
    }

    fn update(&mut self) {
        self.update();
    }
}
