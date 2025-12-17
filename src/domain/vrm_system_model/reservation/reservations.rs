use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use rand::{Rng, seq::IndexedRandom};
use std::{
    collections::{HashMap, hash_map},
    i64,
};

/// TODO Add Comment
#[derive(Debug, Clone)]
pub struct Reservations {
    reservations: HashMap<ReservationId, Box<dyn Reservation>>,
}

impl Reservations {
    pub fn new(reservation: Box<dyn Reservation>) -> Self {
        let key = reservation.get_name();
        let mut map = HashMap::new();
        map.insert(key, reservation);

        Reservations { reservations: map }
    }

    pub fn new_empty() -> Self {
        Reservations { reservations: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.reservations.clear();
    }

    pub fn box_clone(&self, key: &ReservationId) -> Box<dyn Reservation> {
        match self.reservations.get(&key) {
            Some(res) => res.box_clone(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }
    pub fn insert(&mut self, key: ReservationId, reservation: Box<dyn Reservation>) {
        self.reservations.insert(key, reservation);
    }

    pub fn delete_reservation(&mut self, key: &ReservationId) -> Option<(ReservationId, Box<(dyn Reservation + 'static)>)> {
        self.reservations.remove_entry(key)
    }

    pub fn contains_key(&self, key: &ReservationId) -> bool {
        self.reservations.contains_key(key)
    }

    pub fn set_state(&mut self, key: &ReservationId, state: ReservationState) {
        match self.reservations.get_mut(key) {
            Some(res) => res.set_state(state),
            None => log::error!("Get mut reservation (id: {}) was not possible.", key),
        }
    }

    pub fn set_frag_delta(&mut self, key: &ReservationId, frag_delta: f64) {
        match self.reservations.get_mut(key) {
            Some(res) => res.set_frag_delta(frag_delta),
            None => log::error!("Get mut reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get(&self, key: &ReservationId) -> Option<&Box<dyn Reservation>> {
        self.reservations.get(key)
    }

    pub fn get_mut(&mut self, key: &ReservationId) -> Option<&mut Box<dyn Reservation>> {
        self.reservations.get_mut(key)
    }

    pub fn len(&self) -> usize {
        self.reservations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.reservations.is_empty()
    }

    pub fn keys(&self) -> hash_map::Keys<'_, ReservationId, Box<dyn Reservation>> {
        self.reservations.keys()
    }

    pub fn get_random_key(&self) -> Option<ReservationId> {
        let keys: Vec<ReservationId> = self.reservations.keys().cloned().collect();
        let mut rng = rand::rng();

        return keys.choose(&mut rng).cloned();
    }

    pub fn get_random_reservation(&self) -> Option<&Box<dyn Reservation>> {
        self.get_random_key().and_then(|key| self.get(&key))
    }

    pub fn get_reservation_with_first_start_slot(&self) -> Option<&Box<dyn Reservation>> {
        let keys: Vec<ReservationId> = self.reservations.keys().cloned().collect();
        let earliest_start_time: i64 = i64::MAX;
        let mut reservation_of_earliest_start_time = None;

        for key in keys {
            if self.get_assigned_start(&key) < earliest_start_time {
                reservation_of_earliest_start_time = self.get(&key).clone()
            }
        }
        return reservation_of_earliest_start_time;
    }

    pub fn iter(&self) -> hash_map::Iter<'_, ReservationId, Box<dyn Reservation>> {
        self.reservations.iter()
    }

    pub fn get_assigned_end(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_assigned_end(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_assigned_start(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_assigned_start(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_booking_interval_start(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_booking_interval_start(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_booking_interval_end(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_booking_interval_end(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_task_duration(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_task_duration(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_is_moldable(&self, key: &ReservationId) -> bool {
        match self.reservations.get(key) {
            Some(res) => res.is_moldable(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }

    pub fn get_reserved_capacity(&self, key: &ReservationId) -> i64 {
        match self.reservations.get(key) {
            Some(res) => res.get_reserved_capacity(),
            None => panic!("Get reservation (id: {}) was not possible.", key),
        }
    }
}
