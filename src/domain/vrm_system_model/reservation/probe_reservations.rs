use std::collections::HashMap;

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::{Reservation, ReservationState},
        reservation_store::{ReservationId, ReservationStore},
    },
    utils::id::{ComponentId, ShadowScheduleId},
};

/// TODO Probe Reservations are never deleted form ReservationStore,
/// Create cleanup of the ReservationStore
pub struct ProbeReservations {
    pub original_reservation_id: ReservationId,
    pub reservation_store: ReservationStore,
    reservation_ids: HashMap<ReservationId, usize>,
    origin_information: Vec<(Option<ComponentId>, Option<ShadowScheduleId>)>,
    reservation_idx: usize,
}

impl ProbeReservations {
    pub fn new(original_reservation_id: ReservationId, reservation_store: ReservationStore) -> Self {
        ProbeReservations {
            original_reservation_id,
            reservation_store,
            reservation_ids: HashMap::new(),
            origin_information: Vec::new(),
            reservation_idx: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.reservation_ids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.reservation_ids.len()
    }

    /// TODO
    pub fn add_only_reservation(&mut self, reservation: Reservation) {
        let reservation_id = self.reservation_store.add(reservation);
        self.reservation_ids.insert(reservation_id, self.reservation_idx);
        self.origin_information.push((None, None));
        self.reservation_idx += 1;
    }

    pub fn add_probe_reservations(&mut self, probe_reservations: ProbeReservations) {
        if self.original_reservation_id.eq(&probe_reservations.original_reservation_id) {
            for (res_id, idx) in probe_reservations.reservation_ids {
                self.reservation_ids.insert(res_id, self.reservation_idx);
                self.origin_information.push(probe_reservations.origin_information[idx].clone());
                self.reservation_idx += 1;
            }
        } else {
            panic!(
                "ProbeReservations: Add ProbeReservations failed, origin ReservationIds do not match {:?} != {:?}",
                self.original_reservation_id, probe_reservations.original_reservation_id
            );
        }
    }

    pub fn delete_reservation(&mut self, reservation_id: ReservationId) {
        match self.reservation_ids.get(&reservation_id) {
            Some(_) => {
                self.reservation_ids.remove(&reservation_id);
                self.reservation_store.update_state(reservation_id, ReservationState::Deleted);
            }
            None => {
                panic!("ProbeReservations: Delete ReservationId {:?} failed, because the id was not present in ProbeReservations.", reservation_id);
            }
        }
    }

    pub fn update_origin_information(
        &mut self,
        original_res_id: ReservationId,
        component_id: ComponentId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) {
        if original_res_id.eq(&self.original_reservation_id) {
            for (_, idx) in &self.reservation_ids {
                self.origin_information[*idx] = (Some(component_id.clone()), shadow_schedule_id.clone());
            }
        } else {
            panic!(
                "ProbeReservations: ProbeAnswer of original Reservation {:?} was requested form ReservationId {:?}. Signals a improper use of ProbeReservations, which will lead to an unexpected outcome.",
                original_res_id, self.original_reservation_id
            );
        }
    }

    /// Function is only for local scheduler
    /// TODO
    pub fn get_res_id_with_first_start_slot(&self, original_res_id: ReservationId) -> Option<ReservationId> {
        if original_res_id.eq(&self.original_reservation_id) {
            let earliest_start_time: i64 = i64::MAX;

            let mut reservation_of_earliest_start_time: Option<ReservationId> = None;
            for (res_id, _) in &self.reservation_ids {
                if self.reservation_store.get_assigned_start(*res_id) < earliest_start_time {
                    reservation_of_earliest_start_time = Some(*res_id);
                }
            }

            return reservation_of_earliest_start_time;
        }
        panic!(
            "ProbeReservationsGetResWithFistSlotIncorrectUse: ProbeAnswer of original Reservation {:?} was requested form ReservationId {:?}. Signals a improper use of ProbeReservations, which will lead to an unexpected outcome.",
            original_res_id, self.original_reservation_id
        );
    }

    /// Local Scheduler use only
    /// TODO
    pub fn get_ids(&self) -> Vec<ReservationId> {
        self.reservation_ids.iter().map(|(res_id, _)| res_id.clone()).collect()
    }

    pub fn get_origin_information(&self, reservation_id: ReservationId) -> (Option<ComponentId>, Option<ShadowScheduleId>) {
        let idx = self.reservation_ids.get(&reservation_id).unwrap();
        self.origin_information[*idx].clone()
    }
}
