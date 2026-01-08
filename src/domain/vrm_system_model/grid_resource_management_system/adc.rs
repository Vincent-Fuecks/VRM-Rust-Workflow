use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::{AcIContainer, AcIManager, DUMMY_COMPONENT_ID};
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, ComponentId, ReservationName, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;
use std::collections::{HashMap, HashSet};
use std::i64;

/**
 * The main component of the VRM: Administrative domain controller (ADC), the Grid broker.
 * The Grid broker is from the communication point of view at the same time a user ({@link ReservationSubmitter})
 * of the underlying AIs and a resource provider ({@link ExtendedReservationProcessor})
 * for the end user or higher level ADCs. The ADC creates for the user an abstract view
 * of all resources in it's administrative domain i.e. the resources of the registered AIs.
 *
 * Atomic jobs will be send to the underlying AIs according to some optimization strategy
 * (see {@link AIOrder}). Complex workflow jobs will be decomposed and for each sub-job a
 * reservation is searched using a {@link WorkflowScheduler}.
 *
 * In order to keep track of all AIs in the administrative domain, the ADC also provides
 * the {@link AIRegistry} interface.
 *
 * In order to reuse the code needed for {@link ExtendedReservationProcessor} the ADC
 * is a sub-class of {@link AI} and the {@link ADCcore} is it's own {@link AdvanceReservationRMS}.
 * The ADC core object contains the actual Grid broker logic.
 *
 * @see ADCcore
 * @see AI
 * @see Client
 * @see WorkflowScheduler
 */
#[derive(Debug)]
pub struct ADC {
    pub id: AdcId,

    /// Registry of connected AIs, wrapped in AiContainer.
    pub aci_manager: AcIManager,

    /// Internal state tracking which AI holds which reservation.
    pub reservation_store: ReservationStore,

    // Strategy for scheduling complex workflows.
    //pub workflow_scheduler: todo!(),
    /// Configuration: Timeout for commits (in seconds)
    pub commit_timeout: i64,

    pub num_of_slots: i64,

    pub slot_width: i64,

    /// Strategy for selecting AIs for atomic jobs
    //pub selection_strategy: AiSelectionStrategy,
    simulator: Box<dyn SystemSimulator>,
}

impl ADC {
    fn new(
        adc_id: AdcId,
        acis: HashSet<Box<dyn ExtendedReservationProcessor>>,
        reservation_store: ReservationStore,
        commit_timeout: i64,
        simulator: Box<dyn SystemSimulator>,
        num_of_slots: i64,
        slot_width: i64,
    ) -> Self {
        let aci_manager = AcIManager::new(adc_id.clone(), acis, simulator.clone_box(), reservation_store.clone(), num_of_slots, slot_width);

        ADC {
            id: adc_id,
            aci_manager: aci_manager,
            reservation_store: reservation_store,
            commit_timeout: commit_timeout,
            simulator: simulator,
            num_of_slots: num_of_slots,
            slot_width: slot_width,
        }
    }

    fn delete_aci(&mut self, aci: AcI) -> bool {
        log::debug!("ACD {} deletes AcI: {}.", self.id, aci.id);
        return self.delete_aci(aci);
    }

    fn add_aci(&mut self, grid_component: Box<dyn ExtendedReservationProcessor>) -> bool {
        log::debug!("ADC: {} adds AcI: {}", self.id, grid_component.get_id());
        return self.aci_manager.add_aci(
            grid_component,
            self.simulator.clone_box(),
            self.reservation_store.clone(),
            self.num_of_slots,
            self.slot_width,
        );
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

    /**
     * Actually performs the commit of the job at the underlying AI.
     *
     * This method is used internally by {@link #commit(Reservation)}
     * and by the {@link WorkflowScheduler}.
     *
     * @param res Reservation to commit
     * @return a Reservation object containing at least the state {@link ReservationState#COMMITED}
     *         on success or {@link ReservationState#STATE_REJECTED} if something went wrong.
     *
     * @see #commit(Reservation)
     */
    fn commit_at_component(&mut self, reservation_id: ReservationId) -> bool {
        // Find responsible component
        let Some(component_id) = self.reservation_store.get_handler_id(reservation_id) else {
            let name = self.reservation_store.get_name_for_key(reservation_id).unwrap_or(ReservationName::new("Does not exist!"));
            log::error!(
                "ReservationHasNoHandlerId: Committing reservation {name} by ADC: {} failed, \
             because reservation has no assigned handler_id.",
                self.id
            );
            return false;
        };

        // Is dummy task/ "Internal task"
        if component_id == *DUMMY_COMPONENT_ID {
            self.reservation_store.update_state(reservation_id, ReservationState::Committed);
            return true;
        }

        match self.aci_manager.get_component_mut(component_id.clone()) {
            Some(container) => {
                if container.grid_component.commit(reservation_id) {
                    return true;
                } else {
                    container.schedule.delete_reservation(reservation_id);
                    self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                    return false;
                }
            }

            None => {
                log::error!("Component: {component_id} was not found in the ComponentManager.");
                return false;
            }
        }
    }
}

impl ExtendedReservationProcessor for ADC {
    fn get_id(&self) -> ComponentId {
        ComponentId::new(self.id.to_string())
    }

    fn get_total_capacity(&self) -> i64 {
        todo!()
    }

    fn get_total_link_capacity(&self) -> i64 {
        todo!()
    }

    fn get_total_node_capacity(&self) -> i64 {
        todo!()
    }

    fn get_link_resource_count(&self) -> usize {
        todo!()
    }

    fn commit(&mut self, reservation_id: ReservationId) -> bool {
        todo!()
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    fn create_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    fn delete_task(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ReservationId {
        todo!()
    }

    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        todo!()
    }

    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        todo!()
    }

    fn get_satisfaction(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        todo!()
    }

    fn get_system_satisfaction(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        todo!()
    }

    fn probe(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> Reservations {
        todo!()
    }

    fn probe_best(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        comparator: &mut dyn Fn(ReservationId, ReservationId) -> std::cmp::Ordering,
    ) -> Option<ReservationId> {
        todo!()
    }

    fn reserve(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ReservationId {
        todo!()
    }
}
