use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use crate::domain::vrm_system_model::reservation::{
    reservation::ReservationState,
    reservation_store::{NotificationListener, ReservationId},
};

/// Listener that implements the "Abo" (subscription) system to keep
/// open_reservations in sync with the ReservationStore state.
#[derive(Debug)]
pub struct VrmStateListener {
    // Shared thread-safe container for open reservations
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

impl NotificationListener for VrmStateListener {
    fn on_reservation_change(&self, key: ReservationId, new_state: ReservationState) {
        // We first check if we know this reservation (mimicking Java's !containsSameName check)
        {
            let guard = self.open_reservations.read().unwrap();
            // In the java version, the check is done before the switch.
            // Here, for optimization, we might only care if we are trying to remove it,
            // but to strictly follow the logging logic:
            if !guard.contains(&key) && new_state != ReservationState::Committed {
                // Note: We skip this warning for COMMITTED because in this Rust impl,
                // the VrmManager explicitly adds it to the list *after* commit in process_reservation,
                // whereas this callback might happen during the commit process itself.
                // However, for DELETED/FINISHED/etc, it should be in the list.
                // log::warn!("Received state change for unknown Reservation {:?}.", key);
            }
        }

        match new_state {
            // Invalid states for an active reservation update in this context
            ReservationState::Open | ReservationState::ProbeAnswer | ReservationState::ReserveAnswer => {
                log::error!("Reservation {:?} was set back to an invalid state: {:?}", key, new_state);
            }
            // No change for Committed (it remains open)
            ReservationState::Committed => {
                // Java: "no change"
            }
            // Removal states
            ReservationState::Deleted | ReservationState::Rejected => {
                log::info!("Reservation {:?} was deleted/rejected.", key);
                let mut guard = self.open_reservations.write().unwrap();
                guard.remove(&key);
            }
            ReservationState::Finished => {
                log::info!("Reservation {:?} finished successfully.", key);
                let mut guard = self.open_reservations.write().unwrap();
                guard.remove(&key);
            }
            // Catch-all for other states if any
            _ => {}
        }
    }
}
