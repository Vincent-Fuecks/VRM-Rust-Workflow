use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::{AcIContainer, AcIManager};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};

use std::collections::{HashMap, HashSet};
use std::i64;

#[derive(Debug)]
pub struct ADC {
    pub id: AdcId,

    /// Registry of connected AIs, wrapped in AiContainer.
    pub known_ais: AcIManager,

    /// Internal state tracking which AI holds which reservation.
    pub reservation_store: ReservationStore,

    // Strategy for scheduling complex workflows.
    //pub workflow_scheduler: todo!(),
    /// Configuration: Timeout for commits (in seconds)
    pub commit_timeout: i64,

    /// Strategy for selecting AIs for atomic jobs
    //pub selection_strategy: AiSelectionStrategy,
    simulator: Box<dyn SystemSimulator>,
}

impl ADC {
    fn new(adc_id: AdcId, acis: HashSet<AcI>, reservation_store: ReservationStore, commit_timeout: i64, simulator: Box<dyn SystemSimulator>) -> Self {
        let known_acis = AcIManager::new(adc_id.clone(), acis);

        ADC { id: adc_id, known_ais: known_acis, reservation_store: reservation_store, commit_timeout: commit_timeout, simulator: simulator }
    }

    fn deregister_aci(&mut self, aci_id: AciId) -> bool {
        log::debug!("ACD {}, deregisters AcI {}.", self.id, aci_id);

        if self.known_ais.delete_aci(aci_id) {
            return true;
        }
        todo!();
        return false;
    }

    fn register_aci(aci_id: AciId) -> bool {
        todo!()
    }

    fn reserve_commit(reservation_id: ReservationId) {
        todo!()
    }

    fn reserve_probe(reservation_id: ReservationId) {
        todo!()
    }

    fn reserve_reserve(reservation_id: ReservationId) {
        todo!()
    }
}
