use crate::domain::vrm_system_model::{
    reservation::{
        probe_reservations::{ProbeReservationComparator, ProbeReservations},
        reservation::{Reservation, ReservationState},
        reservation_store::ReservationId,
    },
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

        let mut candidates = self.calculate_schedule(id);

        if candidates.is_empty() {
            return candidates;
        }

        let frag_before: f64 = self.get_system_fragmentation();
        if self.is_frag_needed {
            for candidate in candidates.get_mut_reservations() {
                let candidate_id = self.reservation_store.add_probe_reservation(candidate.clone());
                // Tempory Reserve
                self.is_frag_cache_up_to_date = false;
                self.reserve_without_check(candidate_id);

                let frag_delta: f64 = self.get_system_fragmentation() - frag_before;
                candidate.set_frag_delta(frag_delta);

                // Deleate from Slots and ReservationStore
                self.delete_reservation(candidate_id);
                self.reservation_store.delete_probe_reservation(candidate_id);
            }
        }

        return candidates;
    }

    fn probe_best(&mut self, request_id: ReservationId, probe_reservation_comparator: ProbeReservationComparator) -> ProbeReservations {
        let mut probe_reservations = self.probe(request_id);
        return probe_reservations.get_best_probe_reservation(request_id, probe_reservation_comparator);
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
        if probe_reservations.is_empty() {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            return None;
        }

        if probe_reservations.prompt_best(reservation_id, ProbeReservationComparator::ESTReservationCompare) {
            self.is_frag_cache_up_to_date = false;
            self.reserve_without_check(reservation_id);
            return Some(reservation_id);
        } else {
            self.active_reservations.set_state(&reservation_id, ReservationState::Rejected);
            return None;
        }
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
