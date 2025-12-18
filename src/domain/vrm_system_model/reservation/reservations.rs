use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
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

    pub fn set_state(&mut self, id: &ReservationId, state: ReservationState) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_state(state);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn set_frag_delta(&mut self, id: &ReservationId, frag_delta: f64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_frag_delta(frag_delta);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn set_booking_interval_start(&mut self, id: &ReservationId, booking_interval_start: i64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_booking_interval_start(booking_interval_start);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn set_booking_interval_end(&mut self, id: &ReservationId, booking_interval_end: i64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_booking_interval_end(booking_interval_end);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn set_assigned_start(&mut self, id: &ReservationId, assigned_start: i64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_assigned_start(assigned_start);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn set_assigned_end(&mut self, id: &ReservationId, assigned_end: i64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.set_assigned_end(assigned_end);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }

    pub fn len(&self) -> usize {
        self.reservations.len()
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
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_assigned_start(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_assigned_end(&self, id: &ReservationId) -> i64 {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_assigned_end(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_booking_interval_start(&self, id: &ReservationId) -> i64 {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_booking_interval_start(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_booking_interval_end(&self, id: &ReservationId) -> i64 {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_booking_interval_end(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_task_duration(&self, id: &ReservationId) -> i64 {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_task_duration(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_is_moldable(&self, id: &ReservationId) -> bool {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().is_moldable(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn get_reserved_capacity(&self, id: &ReservationId) -> i64 {
        match self.reservation_store.get(*id) {
            Some(res) => res.read().unwrap().get_reserved_capacity(),
            None => panic!("Getting reservation (id: {:?}) was not possible.", id),
        }
    }

    pub fn adjust_capacity(&self, id: &ReservationId, capacity: i64) {
        if let Some(handle) = self.reservation_store.get(*id) {
            let mut res = handle.write().unwrap();
            res.adjust_capacity(capacity);
        } else {
            log::error!("Get reservation (id: {:?}) was not possible.", id)
        }
    }
}
