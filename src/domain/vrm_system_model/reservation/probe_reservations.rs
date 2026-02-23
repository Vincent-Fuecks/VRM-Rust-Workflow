use std::{cmp::Ordering, collections::HashMap};

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::{Reservation, ReservationState, ReservationTrait},
        reservation_store::{ReservationId, ReservationStore},
    },
    utils::id::ProbeReservationId,
};

#[derive(Clone, Debug)]
pub enum ProbeReservationComparator {
    EFTReservationCompare,
    ESTReservationCompare,
}

impl ProbeReservationComparator {
    pub fn compare(&self, a: &Reservation, b: &Reservation) -> Ordering {
        match self {
            ProbeReservationComparator::EFTReservationCompare => {
                let assigned_end0 = a.get_base_reservation().get_assigned_end();
                let assigned_end1 = b.get_base_reservation().get_assigned_end();
                return assigned_end0.partial_cmp(&assigned_end1).unwrap();
            }
            ProbeReservationComparator::ESTReservationCompare => {
                let assigned_start0 = a.get_base_reservation().get_assigned_start();
                let assigned_start1 = b.get_base_reservation().get_assigned_start();
                return assigned_start0.partial_cmp(&assigned_start1).unwrap();
            }
        }
    }
}

/// ProbeReservations are hypotitic Reservations, which are only tracked by this
/// ProbeReservations Object.
/// If the ProbeReservation should replace the actual Reservation use `promote_reservation`
#[derive(Debug, Clone)]
pub struct ProbeReservations {
    pub original_reservation_id: ReservationId,
    pub local_reservation_store: HashMap<ProbeReservationId, Reservation>,
    original_reservation: Reservation,
    reservation_store: ReservationStore,
    reservation_idx: usize,
}

impl ProbeReservations {
    pub fn new(original_reservation_id: ReservationId, reservation_store: ReservationStore) -> Self {
        let original_reservation = reservation_store.get_reservation_snapshot(original_reservation_id).unwrap();

        ProbeReservations {
            original_reservation_id,
            local_reservation_store: HashMap::new(),
            original_reservation,

            reservation_store,
            reservation_idx: 0,
        }
    }

    /// TODO
    pub fn add_reservation(&mut self, reservation: Reservation) {
        let probe_reservation_id = ProbeReservationId::new(format!("{}-{}", reservation.get_name(), self.reservation_idx));

        // Check if Reservation is valid
        if reservation.get_assigned_start() < reservation.get_booking_interval_start()
            || reservation.get_assigned_end() > reservation.get_booking_interval_end()
        {
            log::error!("ProbeReservationIsNotValid");
        }

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

    /// Resets Promption of Reservation to original Reservation.
    pub fn demote(&mut self) {
        let res_base = self.original_reservation.get_base_reservation();
        self.reservation_store.set_booking_interval_start(self.original_reservation_id, res_base.get_booking_interval_start());
        self.reservation_store.set_booking_interval_end(self.original_reservation_id, res_base.get_booking_interval_end());
        self.reservation_store.set_assigned_start(self.original_reservation_id, res_base.get_assigned_start());
        self.reservation_store.set_assigned_end(self.original_reservation_id, res_base.get_assigned_end());
        self.reservation_store.update_state(self.original_reservation_id, res_base.get_state());
    }

    /// Finds in the ProbeReservations, the Reservation, which is accordings to the ProbeReservationComparator
    /// the best Reservation und updates the original Reservation with the information of the ProbeReservation.
    ///
    /// Return:
    /// If promotion was successful true is returned otherwise false is returned.
    pub fn prompt_best(&mut self, original_res_id: ReservationId, comparator: ProbeReservationComparator) -> bool {
        let probe_ids = self.get_best_probe_reservation(original_res_id, comparator).get_ids();

        if probe_ids.len() > 1 {
            panic!("Only one ProbeReservation should be in LocalReservationStore.")
        } else if probe_ids.len() == 0 {
            return false;
        }

        let best_probe_reservation = self.local_reservation_store.remove(probe_ids.get(0).unwrap());

        match best_probe_reservation {
            Some(res) => {
                self.reservation_store.set_booking_interval_start(original_res_id, res.get_booking_interval_start());
                self.reservation_store.set_booking_interval_end(original_res_id, res.get_booking_interval_end());
                self.reservation_store.set_assigned_start(original_res_id, res.get_assigned_start());
                self.reservation_store.set_assigned_end(original_res_id, res.get_assigned_end());
                self.reservation_store.update_state(original_res_id, res.get_state());
                return true;
            }
            None => false,
        }
    }

    /// Finds in the ProbeReservations, the Reservation, which is accordings to the ProbeReservationComparator
    /// the best Reservation.
    ///
    /// Return:
    /// Returns a new ProbeReservation object, which only contains the "best ProbeReservation"
    /// If ProbeReservation is Empty an empty ProbeReservation object is returned.
    pub fn get_best_probe_reservation(&mut self, original_res_id: ReservationId, comparator: ProbeReservationComparator) -> ProbeReservations {
        if self.is_request_valid(original_res_id) {
            if self.local_reservation_store.is_empty() {
                return ProbeReservations::new(original_res_id, self.reservation_store.clone());
            }

            if let Some((best_candiate_id, fist_candidate)) = self.local_reservation_store.iter().next() {
                let mut best_candidate: (ProbeReservationId, Reservation) = (best_candiate_id.clone(), fist_candidate.clone());

                for (candidate_id, res_candidate) in &self.local_reservation_store {
                    if comparator.compare(&best_candidate.1, res_candidate) == Ordering::Greater {
                        best_candidate = (candidate_id.clone(), res_candidate.clone());
                    }
                }

                let mut probe_reservations = ProbeReservations::new(original_res_id, self.reservation_store.clone());
                probe_reservations.add_reservation(best_candidate.1);
                return probe_reservations;
            }
        }

        return ProbeReservations::new(original_res_id, self.reservation_store.clone());
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

    pub fn get_mut_reservations(&mut self) -> Vec<&mut Reservation> {
        self.local_reservation_store.values_mut().map(|value| value).collect()
    }

    /// Checks if request_id and original ReservationId are the same
    fn is_request_valid(&self, test_res_id: ReservationId) -> bool {
        if test_res_id.eq(&self.original_reservation_id) {
            return true;
        } else {
            log::error!(
                "ProbeReservationsGetResWithFistSlotIncorrectUse: ProbeAnswer of original Reservation {:?} was requested form ReservationId {:?}. Signals a improper use of ProbeReservations, which will lead to an unexpected outcome.",
                self.reservation_store.get_name_for_key(test_res_id),
                self.original_reservation_id
            );
            return false;
        }
    }

    // fn promote_reservation(&mut self, probe_reservation_id: ProbeReservationId) -> bool {
    //     if let Some(res_to_prompt) = self.local_reservation_store.get(&probe_reservation_id) {
    //         self.reservation_store.set_booking_interval_start(self.original_reservation_id, res_to_prompt.get_booking_interval_start());
    //         self.reservation_store.set_booking_interval_end(self.original_reservation_id, res_to_prompt.get_booking_interval_end());
    //         self.reservation_store.set_assigned_start(self.original_reservation_id, res_to_prompt.get_assigned_start());
    //         self.reservation_store.set_assigned_end(self.original_reservation_id, res_to_prompt.get_assigned_end());
    //         self.reservation_store.update_state(self.original_reservation_id, res_to_prompt.get_state());
    //         return true;
    //     }

    //     return false;
    // }
}
