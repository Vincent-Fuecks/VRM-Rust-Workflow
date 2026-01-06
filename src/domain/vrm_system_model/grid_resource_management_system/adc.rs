use crate::domain::simulator;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::{AciContainer, AciManager};
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};

use std::collections::HashMap;
use std::i64;

#[derive(Debug)]
pub struct ADC {
    pub id: AdcId,

    /// Registry of connected AIs, wrapped in AiContainer.
    pub known_ais: AciManager,

    /// Internal state tracking which AI holds which reservation.
    pub reservation_store: ReservationStore,

    // Strategy for scheduling complex workflows.
    //pub workflow_scheduler: todo!(),
    /// Configuration: Timeout for commits (in seconds)
    pub commit_timeout: i64,

    /// Strategy for selecting AIs for atomic jobs
    //pub selection_strategy: AiSelectionStrategy,

    /// Counter to assign stable indices to AIs for sorting
    registration_counter: usize,

    /// State for RoundRobin strategy (tracks the index of the next AI to try first)
    next_rr_index: usize,

    simulator: Box<dyn SystemSimulator>,
}

impl ADC {
    fn new(adc_id: AdcId, acis: Vec<AcI>, reservation_store: ReservationStore, commit_timeout: i64, simulator: Box<dyn SystemSimulator>) -> Self {
        let mut registration_counter = 0;

        let mut known_acis = HashMap::new();

        ADC {
            id: adc_id,
            known_ais: known_acis,
            reservation_store: reservation_store,
            commit_timeout: commit_timeout,
            registration_counter: registration_counter,
            next_rr_index: 0,
            simulator: simulator,
        }
    }

    fn deregister_aci(&mut self, aci_id: AciId) -> bool {
        log::debug!("ACD {}, deregisters AcI {}.", self.id, aci_id);

        if self.known_ais.delete_aci(aci_id) {
            return;
        }
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
