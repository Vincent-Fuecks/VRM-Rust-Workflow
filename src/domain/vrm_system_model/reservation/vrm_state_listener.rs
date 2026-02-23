use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::domain::vrm_system_model::{
    reservation::{
        reservation::ReservationState,
        reservation_store::{NotificationListener, ReservationId},
    },
    utils::id::ReservationName,
};

/// Listener that implements the "Abo" (subscription) system to keep
/// open_reservations in sync with the ReservationStore state.
#[derive(Debug)]
pub struct VrmStateListener {
    open_reservations: Arc<RwLock<HashSet<ReservationId>>>,
}

impl VrmStateListener {
    pub fn new(open_reservations: Arc<RwLock<HashSet<ReservationId>>>) -> Self {
        Self { open_reservations }
    }

    pub fn new_empty() -> Self {
        Self { open_reservations: Arc::new(RwLock::new(HashSet::new())) }
    }

    pub fn add(&mut self, reservation_id: ReservationId) -> bool {
        let mut guard = self.open_reservations.write().unwrap();
        guard.insert(reservation_id)
    }
}

// TODO What should happen by State change?
impl NotificationListener for VrmStateListener {
    fn on_reservation_change(
        &mut self,
        reservation_id: ReservationId,
        res_name: ReservationName,
        old_state: ReservationState,
        new_state: ReservationState,
    ) {
        match new_state {
            ReservationState::Open => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::ProbeAnswer => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::ReserveAnswer => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::ProbeReservation => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::ReserveProbeReservation => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::Committed => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
            }
            ReservationState::Deleted => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
                let mut guard = self.open_reservations.write().unwrap();
                guard.remove(&reservation_id);
            }
            ReservationState::Rejected => {
                log::info!("State Change of Reservation ID: {:?} | Name: {:?} | {:?}->{:?}", reservation_id, res_name, old_state, new_state);
                let mut guard = self.open_reservations.write().unwrap();
                guard.remove(&reservation_id);
            }
            ReservationState::Finished => {
                log::info!("Reservation {:?} finished successfully.", reservation_id);
                let mut guard = self.open_reservations.write().unwrap();
                guard.remove(&reservation_id);
            }
        }
    }
}
