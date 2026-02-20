use std::collections::HashMap;

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::{Reservation, ReservationTrait},
        reservation_store::{ReservationId, ReservationStore},
    },
    utils::id::{ComponentId, ProbeReservationId, ShadowScheduleId},
};

/// ProbeReservations are hypotitic Reservations, which are only tracked by this
/// ProbeReservations Object.
/// If the ProbeReservation should replace the actual Reservation use `promote_reservation`
#[derive(Debug, Clone)]
pub struct ProbeReservations {
    pub original_reservation_id: ReservationId,
    pub local_reservation_store: HashMap<ProbeReservationId, Reservation>,
    reservation_store: ReservationStore,
    reservation_idx: usize,
}

impl ProbeReservations {
    pub fn new(original_reservation_id: ReservationId, reservation_store: ReservationStore) -> Self {
        ProbeReservations { original_reservation_id, local_reservation_store: HashMap::new(), reservation_store, reservation_idx: 0 }
    }

    /// TODO
    pub fn add_reservation(&mut self, reservation: Reservation) {
        let probe_reservation_id = ProbeReservationId::new(format!("{}-{}", reservation.get_name(), self.reservation_idx));

        if self.local_reservation_store.insert(probe_reservation_id, reservation).is_some() {
            panic!("Can not add two ProbeReservations with the same name to the local store.");
        }

        self.reservation_idx += 1;
    }

    pub fn add_probe_reservations(&mut self, probe_reservations: ProbeReservations) {
        if probe_reservations.original_reservation_id.eq(&self.original_reservation_id) {
            if self.original_reservation_id.eq(&probe_reservations.original_reservation_id) {
                for (_, res) in probe_reservations.local_reservation_store {
                    self.add_reservation(res);
                }
            } else {
                panic!(
                    "ProbeReservations: Add ProbeReservations failed, origin ReservationIds do not match {:?} != {:?}",
                    self.original_reservation_id, probe_reservations.original_reservation_id
                );
            }
        }
    }

    pub fn promote_probe_res_with_fist_start(&mut self, original_res_id: ReservationId) -> bool {
        if original_res_id.eq(&self.original_reservation_id) {
            let earliest_start_time: i64 = i64::MAX;
            let mut reservation_of_earliest_start_time: Option<ProbeReservationId> = None;

            for (id, res) in &self.local_reservation_store {
                if res.get_assigned_start() < earliest_start_time {
                    reservation_of_earliest_start_time = Some(id.clone());
                }
            }

            match reservation_of_earliest_start_time {
                Some(id) => {
                    let res_to_prompot = self.local_reservation_store.get(&id).unwrap();

                    self.reservation_store.set_booking_interval_start(original_res_id, res_to_prompot.get_booking_interval_start());
                    self.reservation_store.set_booking_interval_end(original_res_id, res_to_prompot.get_booking_interval_end());
                    self.reservation_store.set_assigned_start(original_res_id, res_to_prompot.get_assigned_start());
                    self.reservation_store.set_assigned_end(original_res_id, res_to_prompot.get_assigned_end());
                    self.reservation_store.update_state(original_res_id, res_to_prompot.get_state());
                    return true;
                }
                None => {
                    return false;
                }
            }
        }
        panic!(
            "ProbeReservationsGetResWithFistSlotIncorrectUse: ProbeAnswer of original Reservation {:?} was requested form ReservationId {:?}. Signals a improper use of ProbeReservations, which will lead to an unexpected outcome.",
            original_res_id, self.original_reservation_id
        );
    }

    pub fn get_ids(&self) -> Vec<ProbeReservationId> {
        self.local_reservation_store.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.local_reservation_store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.local_reservation_store.is_empty()
    }
}
