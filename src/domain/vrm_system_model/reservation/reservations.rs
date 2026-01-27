use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use rand::seq::IndexedRandom;
use std::collections::hash_set;
use std::{collections::HashSet, i64};

/// TODO Add Comment
#[derive(Debug, Clone)]
pub struct Reservations {
    reservations: HashSet<ReservationId>,
    reservation_store: ReservationStore,
}

impl Reservations {
    pub fn new_empty(reservation_store: ReservationStore) -> Self {
        Reservations { reservations: HashSet::new(), reservation_store }
    }

    pub fn clear(&mut self) {
        self.reservations = HashSet::new();
    }

    // TODO maybe insert Real Reservation into store?
    // TODO Change handler_id?
    pub fn insert(&mut self, id: ReservationId) {
        self.reservations.insert(id);
    }

    // TODO maybe del Real Reservation into store?
    // TODO Change handler_id?
    pub fn delete_reservation(&mut self, id: &ReservationId) -> bool {
        if self.reservations.remove(id) {
            self.set_state(id, ReservationState::Deleted);
            return true;
        }
        return false;
    }

    pub fn contains_key(&self, id: &ReservationId) -> bool {
        self.reservations.contains(id)
    }

    // TODO Should we send an update to each Component with interest?! Update_state is doing this.
    pub fn set_state(&mut self, id: &ReservationId, new_state: ReservationState) {
        self.reservation_store.update_state(id.clone(), new_state);
    }

    pub fn set_frag_delta(&mut self, id: &ReservationId, frag_delta: f64) {
        self.reservation_store.set_frag_delta(id.clone(), frag_delta);
    }

    pub fn set_booking_interval_start(&mut self, id: &ReservationId, booking_interval_start: i64) {
        self.reservation_store.set_booking_interval_start(id.clone(), booking_interval_start);
    }

    pub fn set_booking_interval_end(&mut self, id: &ReservationId, booking_interval_end: i64) {
        self.reservation_store.set_booking_interval_end(id.clone(), booking_interval_end);
    }

    pub fn set_assigned_start(&mut self, id: &ReservationId, assigned_start: i64) {
        self.reservation_store.set_assigned_start(id.clone(), assigned_start);
    }

    pub fn set_assigned_end(&mut self, id: &ReservationId, assigned_end: i64) {
        self.reservation_store.set_assigned_end(id.clone(), assigned_end);
    }

    pub fn len(&self) -> usize {
        self.reservations.len()
    }

    // Log all reservations
    pub fn dump_reservation(&self) {
        log::error!("=== RESERVATION RESERVATION(s) DUMP ({} entries) ===", self.reservations.len());
        for reservation_id in &self.reservations {
            log::error!("  -> ID: {:?} | Name: {:?}", reservation_id, self.reservation_store.get_name_for_key(reservation_id.clone()));
        }
        log::error!("=== END OF DUMP ===");
    }

    pub fn is_empty(&self) -> bool {
        self.reservations.is_empty()
    }

    pub fn get_random_id(&self) -> Option<ReservationId> {
        let ids: Vec<ReservationId> = self.reservations.iter().into_iter().cloned().collect();
        let mut rng = rand::rng();

        return ids.choose(&mut rng).cloned();
    }

    pub fn get_id_with_first_start_slot(&self) -> Option<ReservationId> {
        let ids: Vec<ReservationId> = self.reservations.iter().into_iter().cloned().collect();
        let earliest_start_time: i64 = i64::MAX;
        let mut reservation_of_earliest_start_time = None;

        for id in ids {
            if self.get_assigned_start(&id) < earliest_start_time {
                reservation_of_earliest_start_time = Some(id);
            }
        }
        return reservation_of_earliest_start_time;
    }

    pub fn iter(&self) -> hash_set::Iter<'_, ReservationId> {
        self.reservations.iter()
    }

    pub fn get_assigned_start(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_assigned_start(id.clone())
    }

    pub fn get_assigned_end(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_assigned_end(id.clone())
    }

    pub fn get_booking_interval_start(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_booking_interval_start(id.clone())
    }

    pub fn get_booking_interval_end(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_booking_interval_end(id.clone())
    }

    pub fn get_task_duration(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_task_duration(id.clone())
    }

    pub fn get_is_moldable(&self, id: &ReservationId) -> bool {
        self.reservation_store.is_moldable(id.clone())
    }

    pub fn get_reserved_capacity(&self, id: &ReservationId) -> i64 {
        self.reservation_store.get_reserved_capacity(id.clone())
    }

    pub fn adjust_capacity(&self, id: &ReservationId, capacity: i64) {
        self.reservation_store.adjust_capacity(id.clone(), capacity);
    }

    pub fn get_reservation_snapshot(&mut self, id: &ReservationId) -> Reservation {
        self.reservation_store.get_reservation_snapshot(*id).unwrap()
    }
}
