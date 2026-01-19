use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_component;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::{self, AcIContainer, AcIManager, DUMMY_COMPONENT_ID};
use crate::domain::vrm_system_model::grid_resource_management_system::aci_order::AcIOrder;
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::grid_resource_management_system::order_grid_component_res_database::OrderGridComponentResDatabase;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, ComponentId, ReservationName, RouterId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;
use crate::domain::vrm_system_model::workflow::workflow::Workflow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::i64;
use std::sync::Arc;

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

    pub aci_order: AcIOrder,

    // Strategy for scheduling complex workflows.
    //pub workflow_scheduler: todo!(),
    /// Configuration: Timeout for commits (in seconds)
    pub commit_timeout: i64,

    pub num_of_slots: i64,

    pub slot_width: i64,

    /// Strategy for selecting AIs for atomic jobs
    //pub selection_strategy: AiSelectionStrategy,
    simulator: Arc<dyn SystemSimulator>,
}

impl ADC {
    fn new(
        adc_id: AdcId,
        acis: HashSet<Box<dyn ExtendedReservationProcessor>>,
        reservation_store: ReservationStore,
        aci_order: AcIOrder,
        commit_timeout: i64,
        simulator: Arc<dyn SystemSimulator>,
        num_of_slots: i64,
        slot_width: i64,
    ) -> Self {
        let aci_manager = AcIManager::new(adc_id.clone(), acis, simulator.clone_box().into(), reservation_store.clone(), num_of_slots, slot_width);

        ADC {
            id: adc_id,
            aci_manager: aci_manager,
            reservation_store: reservation_store,
            aci_order: aci_order,
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
            self.simulator.clone_box().into(),
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

    /**
     * Actually performs the reservation of the job at the underlying AI, uses
     * the first AI (according to AI order) which accepts the reservation.
     *
     * This method is used internally by {@link #reserve(Reservation, String)}
     * and by the {@link WorkflowScheduler}.
     *
     * @param res
     *            The reservation to reserve
     * @param shadowScheduleID
     *            If the method should be applied to a shadow schedule, it's ID
     *            or null otherwise for the normal schedule
     * @param aiPerReservation
     *            Container for AI-Reservation mapping. Most likely the
     *            {@link #aiPerReservation} field of this object, but may also
     *            be used with another container by WorkflowScheduler
     * @return a Reservation object containing at least the state {@link ReservationState#STATE_RESERVEANSWER}
     *         on success or {@link ReservationState#STATE_REJECTED} if something went wrong.
     *
     * @see #reserve(Reservation, String)
     * @see #submitJobAtAIBestMatch(NodeReservation, AIReservationDatabase, String, Comparator)
     * @see #requestOrder
     */
    pub fn submit_task_at_first_grid_component(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
    ) -> ReservationId {
        // Wrong order
        for component_id in self.aci_manager.get_ordered_acis(self.aci_order) {
            // TODO Change, if communication with aci is over the network
            let res = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.aci_manager.can_handel(component_id.clone(), res) {
                let reserve_res_id = self.aci_manager.reserve(component_id.clone(), reservation_id, shadow_schedule_id.clone());

                if self.reservation_store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer) {
                    // Register new schedule Sub-Task
                    // Update grid_component_res_database for rollback and for ADC to keep track
                    if grid_component_res_database.contains_key(&reserve_res_id) {
                        log::error!(
                            "ErrorReservationWasReservedInMultipleGridComponents: The reservation {:?} was multiple times to the GirdComponent {} submitted.",
                            self.reservation_store.get_name_for_key(reserve_res_id),
                            component_id
                        );
                        grid_component_res_database.insert(reserve_res_id, component_id.clone());
                    } else {
                        grid_component_res_database.insert(reserve_res_id, component_id.clone());
                    }
                    self.aci_manager.reserve_without_check(component_id.clone(), reserve_res_id);

                    if !self.reservation_store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer) {
                        log::error!("Reserve of reservation {:?} in local schedule copy of Grid Component {} failed.", reserve_res_id, component_id);
                    }

                    return reserve_res_id;
                }
            }
        }
        self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
        return reservation_id;
    }

    /**
     * Actually performs the reservation of the job at the underlying AI, probes all
     * AIs and then picks the best candidate according to the given order.
     *
     * This method is used internally by {@link #reserve(Reservation, String)}
     * and by the {@link WorkflowScheduler}.
     *
     * @param res
     *            The reservation to reserve
     * @param shadowScheduleID
     *            If the method should be applied to a shadow schedule, it's ID
     *            or null otherwise for the normal schedule
     * @param aisPerReservation
     *            Container for AI-Reservation mapping. Most likely the
     *            {@link #aiPerReservation} field of this object, but may also
     *            be used with another container by WorkflowScheduler
     * @param reservationOrder
     *            Order to be used to sort probe answer and finally book reservation candidates.
     *            If for two reservations are equal according to this order ({@link Comparator#compare(Object, Object)}==0)
     *            the normal order of the AIs ({@link #requestOrder}) will be used to sort them.
     * @return a Reservation object containing at least the state {@link ReservationState#STATE_RESERVEANSWER}
     *         on success or {@link ReservationState#STATE_REJECTED} if something went wrong.
     *
     * @see #reserve(Reservation, String)
     * @see #submitJobAtAIFirstMatch(Reservation, String, AIReservationDatabase)
     */
    pub fn submit_task_at_best_aci<F>(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
        reservation_order: F,
    ) -> Option<ReservationId>
    where
        F: Fn(ReservationId, ReservationId) -> Ordering + 'static,
    {
        let mut order_grid_component_res_database = OrderGridComponentResDatabase::new(reservation_order, self.aci_order.get_comparator());

        for component_id in self.aci_manager.get_random_ordered_acis() {
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.aci_manager.can_handel(component_id.clone(), res_snapshot) {
                let probe_reservations = self
                    .aci_manager
                    .get_component_mut(component_id.clone())
                    .unwrap()
                    .grid_component
                    .probe(reservation_id, shadow_schedule_id.clone());

                // Do not trust answer of lower GridComponent
                for prob_reservation_id in probe_reservations.iter() {
                    if self.reservation_store.get_assigned_start(*prob_reservation_id)
                        < self.reservation_store.get_booking_interval_start(*prob_reservation_id)
                        || self.reservation_store.get_assigned_end(*prob_reservation_id)
                            > self.reservation_store.get_booking_interval_end(*prob_reservation_id)
                    {
                        log::error!("Invalid Answer.");
                    }
                }

                order_grid_component_res_database.put_all(probe_reservations, component_id.clone());
            }
        }

        // Choose reservation candidate with EFT and reserve it
        for reservation_id in order_grid_component_res_database.sorted_key_set(&self.aci_manager) {
            let component_id = order_grid_component_res_database.store.get(&reservation_id).unwrap();

            let candidate_id = self.aci_manager.reserve(component_id.clone(), reservation_id, None);

            if self.reservation_store.is_reservation_state_at_least(candidate_id, ReservationState::ReserveAnswer) {
                // Register new schedule Sub-Task
                // Update grid_component_res_database for rollback and for ADC to keep track

                if grid_component_res_database.contains_key(&candidate_id) {
                    log::error!(
                        "ErrorReservationWasReservedInMultipleGridComponents: The reservation {:?} was multiple times to the GirdComponent {} submitted.",
                        self.reservation_store.get_name_for_key(candidate_id),
                        component_id
                    );
                    grid_component_res_database.insert(candidate_id, component_id.clone());
                } else {
                    grid_component_res_database.insert(candidate_id, component_id.clone());
                }

                // Update local schedule
                self.aci_manager.reserve_without_check(component_id.clone(), candidate_id);

                if self.reservation_store.is_reservation_state_at_least(candidate_id, ReservationState::ReserveAnswer) {
                    log::error!("Reserve of reservation {:?} in local schedule of GridComponent {:?} failed.", candidate_id, component_id);
                }
                return Some(candidate_id);
            }
        }

        return None;
    }

    /**
     * Actually performs the deletion of the job at the underlying AI. This
     * methods allows also to provide the AI, this job was booked at.
     *
     * This method is used internally by {@link #deleteJob(Reservation, String)}
     * and by the {@link WorkflowScheduler}.
     *
     * @param res Reservation to delete
     * @param shadowScheduleID
     *            If the method should be applied to a shadow schedule, it's ID
     *            or null otherwise for the normal schedule
     * @param ai The AI this reservation was booked at.           
     * @return a Reservation object containing the state {@link ReservationState#DELETED}
     *         on success or {@link ReservationState#STATE_REJECTED} if something went wrong.
     *
     * @see #deleteJob(Reservation, String)
     */
    pub fn delete_task_at_component(
        &mut self,
        component_id: ComponentId,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) {
        todo!()
    }

    pub fn register_workflow_subtasks(&mut self, workflow: &Workflow, grid_component_res_database: &HashMap<ReservationId, ComponentId>) {
        todo!()
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

    fn get_router_list(&self) -> Vec<RouterId> {
        let component_router_list = self
            .aci_manager
            .get_random_ordered_acis()
            .into_iter()
            .flat_map(|component_id| self.aci_manager.get_component_router_list(component_id))
            .collect();

        return component_router_list;
    }

    fn can_handel(&self, res: Reservation) -> bool {
        // TODO Can one of the GridComponents handle the request?
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
