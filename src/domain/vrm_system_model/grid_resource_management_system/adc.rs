use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::order_res_vrm_component_database::OrderResVrmComponentDatabase;
use crate::domain::vrm_system_model::grid_resource_management_system::scheduler::workflow_scheduler::WorkflowScheduler;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_manager::{DUMMY_COMPONENT_ID, VrmComponentManager};
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_order::VrmComponentOrder;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{AdcId, ComponentId, ReservationName, RouterId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::i64;
use std::sync::Arc;

/// The **Administrative Domain Controller (ADC)** acts as the central Grid Broker within the VRM system.
///
/// It operates in a dual capacity:
/// 1. **Consumer**: Acts as a reservation submitter to underlying **VrmComponentManager**.
/// 2. **Provider**: Functions as an `VrmComponent` for end-users or higher-level ADCs.
///
/// The ADC provides an abstracted view of all resources within its administrative domain. It handles
/// **Atomic Jobs** by delegating them to the most suitable VrmComponent based on an optimization strategy,
/// and **Complex Workflows** by decomposing them into sub-jobs via the `WorkflowScheduler`.
#[derive(Debug)]
pub struct ADC {
    pub id: AdcId,
    simulator: Arc<dyn SystemSimulator>,
    pub reservation_store: ReservationStore,

    /// Registry and management interface for all connected VrmComponents in the domain.
    pub manager: VrmComponentManager,

    /// Logic for decomposing and scheduling workflows.
    pub workflow_scheduler: Box<dyn WorkflowScheduler>,

    /// Defines the ordering and selection priority for underlying VrmComponents.
    pub vrm_component_order: VrmComponentOrder,

    /// The maximum duration (in seconds) allowed for a reservation to move from 'Reserved' to 'Committed'
    pub commit_timeout: i64,

    /// Total number of discrete scheduling slots available across the domain.
    pub num_of_slots: i64,

    /// The duration of a single resource slot.
    pub slot_width: i64,
}

impl ADC {
    fn new(
        adc_id: AdcId,
        vrm_components_set: HashSet<Box<dyn VrmComponent>>,
        reservation_store: ReservationStore,
        workflow_scheduler: Box<dyn WorkflowScheduler>,
        vrm_component_order: VrmComponentOrder,
        commit_timeout: i64,
        simulator: Arc<dyn SystemSimulator>,
        num_of_slots: i64,
        slot_width: i64,
    ) -> Self {
        let vrm_component_manager = VrmComponentManager::new(
            adc_id.clone(),
            vrm_components_set,
            simulator.clone_box().into(),
            reservation_store.clone(),
            num_of_slots,
            slot_width,
        );

        ADC {
            id: adc_id,
            manager: vrm_component_manager,
            workflow_scheduler: workflow_scheduler,
            reservation_store: reservation_store,
            vrm_component_order: vrm_component_order,
            commit_timeout: commit_timeout,
            simulator: simulator,
            num_of_slots: num_of_slots,
            slot_width: slot_width,
        }
    }

    // TODO Should work with GridComponent
    /// Removes an VrmComponent from the registry based on its unique identifier.
    fn delete_vrm_component(&mut self, vrm_component: Box<dyn VrmComponent>) -> bool {
        log::debug!("ACD {} deletes VrmComponent {}", self.id, vrm_component.get_id());
        return self.manager.delete_vrm_component(vrm_component.get_id());
    }

    /// Adds a new `GridComponent` to the domain and initializes its local schedule view.
    fn add_vrm_component(&mut self, vrm_component: Box<dyn VrmComponent>) -> bool {
        log::debug!("ADC: {} adds AcI: {}", self.id, vrm_component.get_id());
        return self.manager.add_vrm_component(
            vrm_component,
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

    /// Performs the commit operation at the specific underlying component.
    ///
    /// This is used internally for both atomic tasks and sub-tasks within a workflow.
    /// If the component is a dummy/internal component, the state is updated locally.
    /// Returns `true` if the component successfully committed the reservation.
    pub fn commit_at_component(&mut self, reservation_id: ReservationId) -> bool {
        // Find responsible component
        let Some(component_id) = self.manager.get_handler_id(reservation_id) else {
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

        match self.manager.get_component_mut(component_id.clone()) {
            Some(container) => {
                if container.vrm_component.commit(reservation_id) {
                    return true;
                } else {
                    // If commit fails, clean up local schedule and global mapping
                    container.schedule.delete_reservation(reservation_id);
                    self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                    // TODO Also remove from VrmComponentManager tracking?
                    return false;
                }
            }

            None => {
                log::error!("Component: {component_id} was not found in the ComponentManager.");
                return false;
            }
        }
    }

    /// Submits a task to the first VrmComponent that accepts the reservation based on the defined `VrmComponentOrder`.
    ///
    /// Updates the `TODO` to maintain the mapping between the
    /// reservation and the component that accepted it.
    pub fn submit_task_at_first_grid_component(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
    ) -> ReservationId {
        // Wrong order
        for component_id in self.manager.get_ordered_vrm_components(self.vrm_component_order) {
            // TODO Change, if communication with aci is over the network
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.manager.can_handel(component_id.clone(), res_snapshot) {
                let reserve_res_id = self.manager.reserve(component_id.clone(), reservation_id, shadow_schedule_id.clone());

                if self.reservation_store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer) {
                    // Register new schedule Sub-Task
                    // Update grid_component_res_database for rollback and for ADC to keep track
                    // Update local WorkflowScheduler Log (for rollback and later merge)
                    if grid_component_res_database.contains_key(&reserve_res_id) {
                        log::error!(
                            "ErrorReservationWasReservedInMultipleGridComponents: The reservation {:?} was multiple times to the GirdComponent {} submitted.",
                            self.reservation_store.get_name_for_key(reserve_res_id),
                            component_id
                        );
                    }
                    grid_component_res_database.insert(reserve_res_id, component_id.clone());

                    // Update VrmComponent's local view (schedule) of the underlying VrmComponents
                    self.manager.reserve_without_check(component_id.clone(), reserve_res_id);

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

    /// Probes all available VrmComponents and selects the best candidate based on the provided comparison function.
    ///
    /// This implements a "Best Fit" strategy, useful for optimizing resource utilization or
    /// meeting Earliest Finish Time (EFT) constraints.
    pub fn submit_task_at_best_vrm_component<F>(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
        reservation_order: F,
    ) -> Option<ReservationId>
    where
        F: Fn(ReservationId, ReservationId) -> Ordering + 'static,
    {
        let mut order_grid_component_res_database = OrderResVrmComponentDatabase::new(reservation_order, self.vrm_component_order.get_comparator());

        for component_id in self.manager.get_random_ordered_vrm_components() {
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.manager.can_handel(component_id.clone(), res_snapshot) {
                let probe_reservations =
                    self.manager.get_component_mut(component_id.clone()).unwrap().vrm_component.probe(reservation_id, shadow_schedule_id.clone());

                // Do not trust answer of lower GridComponent
                // Validation of probe answers
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
        for reservation_id in order_grid_component_res_database.sorted_key_set(&self.manager) {
            let component_id = order_grid_component_res_database.store.get(&reservation_id).unwrap();

            let candidate_id = self.manager.reserve(component_id.clone(), reservation_id, None);

            if self.reservation_store.is_reservation_state_at_least(candidate_id, ReservationState::ReserveAnswer) {
                // Register new schedule Sub-Task
                // Update grid_component_res_database for rollback and for ADC to keep track
                // Update Transaction Log
                if grid_component_res_database.contains_key(&candidate_id) {
                    log::error!(
                        "ErrorReservationWasReservedInMultipleGridComponents: The reservation {:?} was multiple times to the GirdComponent {} submitted.",
                        self.reservation_store.get_name_for_key(candidate_id),
                        component_id
                    );
                }
                grid_component_res_database.insert(candidate_id, component_id.clone());

                // Update local schedule
                self.manager.reserve_without_check(component_id.clone(), candidate_id);

                if self.reservation_store.is_reservation_state_at_least(candidate_id, ReservationState::ReserveAnswer) {
                    log::error!("Reserve of reservation {:?} in local schedule of GridComponent {:?} failed.", candidate_id, component_id);
                }
                return Some(candidate_id);
            }
        }

        return None;
    }

    /// Deletes a task from the underlying component and cleans up the associated local schedule.
    pub fn delete_task_at_component(
        &mut self,
        component_id: ComponentId,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) {
        todo!()
    }
    /// Finalizes the allocation of subtasks for a completed workflow scheduling process.
    ///
    /// This merges the temporary transaction map created by teh scheduler into the global system state.
    pub fn register_workflow_subtasks(&mut self, workflow_res_id: ReservationId, grid_component_res_database: &HashMap<ReservationId, ComponentId>) {
        self.manager.register_workflow_allocation(workflow_res_id, grid_component_res_database.clone());
    }
}

impl VrmComponent for ADC {
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
            .manager
            .get_random_ordered_vrm_components()
            .into_iter()
            .flat_map(|component_id| self.manager.get_component_router_list(component_id))
            .collect();

        return component_router_list;
    }

    fn can_handel(&self, res: Reservation) -> bool {
        for component_id in self.manager.get_random_ordered_vrm_components() {
            if self.manager.can_handel(component_id, res.clone()) {
                return true;
            }
        }
        false
    }

    fn commit(&mut self, reservation_id: ReservationId) -> bool {
        if self.reservation_store.is_workflow(reservation_id) {
            let sub_ids = self.workflow_scheduler.get_sub_ids(reservation_id);

            for res_id in sub_ids {
                let component_answer = self.commit_at_component(res_id);
                let state = self.reservation_store.get_state(res_id);

                // Check if this specific sub-component succeeded
                if state != ReservationState::Committed || !component_answer {
                    log::error!("Sub-task {:?} failed in workflow {:?}", res_id, reservation_id);
                    self.workflow_scheduler.handle_failure(reservation_id);
                    return false;
                }
            }

            self.workflow_scheduler.finalize_commit(reservation_id);
            return true;
        } else {
            // Non-workflow atomic job
            return self.commit_at_component(reservation_id);
        }
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
        if self.reservation_store.is_workflow(reservation_id) {
            // TODO
            todo!();
        }

        if let Some(component_id) = self.manager.get_handler_id(reservation_id) {
            self.delete_task_at_component(component_id, reservation_id, shadow_schedule_id);
            return reservation_id;
        } else {
            log::error!("ADC Delete: No handler found for reservation {:?}", reservation_id);
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            return reservation_id;
        }
    }

    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        self.manager.get_load_metric(start, end, shadow_schedule_id)
    }

    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        self.manager.get_load_metric(start, end, shadow_schedule_id)
    }

    fn get_satisfaction(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        self.manager.get_satisfaction(start, end, shadow_schedule_id)
    }

    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        self.manager.get_simulation_load_metric(shadow_schedule_id)
    }

    fn get_system_satisfaction(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        self.manager.get_system_satisfaction(shadow_schedule_id)
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
        if self.reservation_store.is_workflow(reservation_id) {
            // TODO
            todo!();
        } else {
            // Atomic Job
            let mut transaction_map = HashMap::new();
            // Try to reserve
            let res_id = self.submit_task_at_first_grid_component(reservation_id, shadow_schedule_id, &mut transaction_map);

            // If successful, register the allocation (Merge Transaction)
            if self.reservation_store.is_reservation_state_at_least(res_id, ReservationState::ReserveAnswer) {
                if let Some(comp_id) = transaction_map.get(&res_id) {
                    self.manager.register_allocation(res_id, comp_id.clone());
                }
            }
            return res_id;
        }
    }
}
