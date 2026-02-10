use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_order::VrmComponentOrder;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_registry::vrm_component_proxy::VrmComponentProxy;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use crate::domain::vrm_system_model::reservation::probe_reservations::ProbeReservations;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule::schedule_context::SlottedScheduleContext;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::utils::id::RouterId;
use crate::domain::vrm_system_model::utils::id::{AdcId, ComponentId, ShadowScheduleId, SlottedScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;
use crate::domain::vrm_system_model::utils::statistics::ANALYTICS_TARGET;
use lazy_static::lazy_static;
use rand::rng;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

lazy_static! {
    pub static ref DUMMY_COMPONENT_ID: ComponentId = ComponentId::new("ADC INTERNAL JOB");
}

// TODO Functions must be synchronized with the AcIs
// TODO Old Java Version contained all resources and enabled access to them looks like this is now not necessary

/// Container holds a VrmComponents (**AcI** or **ADC**) instance and metadata required for sorting and management.
#[derive(Debug)]
pub struct VrmComponentContainer {
    // Contains a AcI or ADC
    pub vrm_component: Box<dyn VrmComponent + Send>,

    reservation_store: ReservationStore,

    // TODO Should the schedule get a separated ReservationStore? Currently GridComponent and schedule have the same.
    // AKA SlottedSchedule
    pub schedule: Box<dyn Schedule>,

    /// The sequence number assigned at registration time, used for stable sorting.
    pub registration_index: usize,

    /// A counter of how many times operations on this VrmComponent have failed.
    pub failures: u32,

    /// The total bandwidth available on all links of the VrmComponent (does not mean free capacity).
    pub total_link_capacity: i64,

    /// The number of distinct link resources of the VrmComponent.
    pub link_resource_count: usize,
}

impl VrmComponentContainer {
    pub fn new(
        vrm_component: Box<dyn VrmComponent + Send>,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
        registration_index: usize,
        number_of_real_slots: i64,
        slot_width: i64,
        total_link_capacity: i64,
        link_resource_count: usize,
    ) -> Self {
        let component_id = vrm_component.get_id();
        // TODO Add Option for different schedule
        let scheduler_id = SlottedScheduleId::new(format!("Scheduler of VrmComponent: {:?}", component_id));

        let total_capacity = vrm_component.get_total_capacity();

        let slotted_schedule_ctx = SlottedScheduleContext::new(
            scheduler_id,
            simulator.get_current_time_in_s(),
            number_of_real_slots,
            slot_width,
            total_capacity,
            false,
            reservation_store.clone(),
        );

        let schedule = Box::new(SlottedSchedule::new(slotted_schedule_ctx, total_capacity, reservation_store.clone(), simulator));

        Self { vrm_component, reservation_store, schedule, registration_index, total_link_capacity, link_resource_count, failures: 0 }
    }

    pub fn can_handel(&self, res: Reservation) -> bool {
        self.vrm_component.can_handel(res)
    }

    pub fn get_router_list(&self) -> Vec<RouterId> {
        self.vrm_component.get_router_list()
    }
}

/// Manages a collection of **VrmComponents (ADCs and/or AcIs)** for a specific **ADC**.
///
/// The `VrmComponentManager` acts as a central registry and aggregator for distributed resources. It handles:
/// * Registration and deregistration of VrmComponents.
/// * Aggregation of system-wide metrics (Satisfaction, Load).
/// * Retrieval of VrmComponents based on specific ordering strategies (Random, Load-balanced, etc.).
///
/// # Distributed Context & Synchronization
///
/// This manager operates within a distributed Grid/VRM system. While `VrmComponentManager` provides a local view
/// of the resources, operations performed on the contained `VrmComponents` objects involve network communication
/// with remote entities (ADCs local, AcIs remote). Callers should assume that state changes (like load updates)
/// require synchronization with the remote AcIs.
#[derive(Debug)]
pub struct VrmComponentManager {
    /// The ID of the ADC owning this manager.
    pub adc_id: AdcId,

    /// Map of `VrmComponentId` to their container wrappers.
    pub vrm_components: HashMap<ComponentId, VrmComponentContainer>,

    // --- Reservation Tracking (The "Who has What") ---
    /// Maps a `ReservationId` (Atomic Job or Workflow Subtask) to the `VrmComponentId` that handles it.
    pub res_to_vrm_component: HashMap<ReservationId, ComponentId>,

    pub committed_reservations: HashMap<ReservationId, ComponentId>,

    pub not_committed_reservations: HashMap<ReservationId, ComponentId>,

    pub shadow_schedule_reservations: HashMap<ShadowScheduleId, (HashMap<ReservationId, ComponentId>, ReservationStore)>,

    /// Maps a `WorkflowId` (Parent) to a list of its sub-reservations (Nodes and Links).
    pub workflow_subtasks: HashMap<ReservationId, Vec<ReservationId>>,

    /// Maps a Subtask `ReservationId` back to its Parent `WorkflowId`.
    pub reverse_workflow_subtasks: HashMap<ReservationId, ReservationId>,

    /// The aggregated sum of link capacities of all registered AcIs (does not mean free capacity).
    pub total_link_capacity: i64,

    /// The aggregated sum distinct link resources of all registered AcIs.
    pub link_resource_count: usize,

    /// Monotonic counter used to assign `registration_index` to new VrmComponentContainer's.
    registration_counter: usize,

    /// Is used to create an empty Reservations struct as return value for an unsuccessful probe request
    pub reservation_store: ReservationStore,

    pub simulator: Arc<dyn SystemSimulator>,
}

impl VrmComponentManager {
    pub fn new(
        adc_id: AdcId,
        vrm_components_list: Vec<VrmComponentProxy>,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
        number_of_real_slots: i64,
        slot_width: i64,
    ) -> Self {
        let mut vrm_components = HashMap::with_capacity(vrm_components_list.len());
        let mut registration_counter = 0;
        let mut manager_total_link_capacity = 0;
        let mut manager_link_resource_count = 0;

        for vrm_component in vrm_components_list {
            let component_id = vrm_component.get_id().clone();
            let total_link_capacity = vrm_component.get_total_link_capacity();
            let link_resource_count = vrm_component.get_link_resource_count();

            manager_total_link_capacity += total_link_capacity;
            manager_link_resource_count += link_resource_count;

            let container = VrmComponentContainer::new(
                Box::new(vrm_component),
                simulator.clone_box().into(),
                reservation_store.clone(),
                registration_counter,
                number_of_real_slots,
                slot_width,
                total_link_capacity,
                link_resource_count,
            );

            registration_counter += 1;
            vrm_components.insert(component_id, container);
        }

        VrmComponentManager {
            adc_id,
            vrm_components,
            res_to_vrm_component: HashMap::new(),
            committed_reservations: HashMap::new(),
            not_committed_reservations: HashMap::new(),
            shadow_schedule_reservations: HashMap::new(),
            workflow_subtasks: HashMap::new(),
            reverse_workflow_subtasks: HashMap::new(),
            total_link_capacity: manager_total_link_capacity,
            link_resource_count: manager_link_resource_count,
            registration_counter,
            reservation_store: reservation_store.clone(),
            simulator: simulator.clone(),
        }
    }

    pub fn get_vrm_component_container_mut(&mut self, component_id: ComponentId) -> &mut VrmComponentContainer {
        match self.vrm_components.get_mut(&component_id) {
            Some(container) => container,
            None => panic!(
                "ErrorFailedToGetVrmComponentContainer: In the VrmComponentManager of ADC {}, was the ComponentId {} not found.",
                self.adc_id,
                component_id.clone()
            ),
        }
    }

    pub fn get_vrm_component_container(&mut self, component_id: ComponentId) -> &VrmComponentContainer {
        match self.vrm_components.get(&component_id) {
            Some(container) => container,
            None => panic!(
                "ErrorFailedToGetVrmComponentContainer: In the VrmComponentManager of ADC {}, was the ComponentId {} not found.",
                self.adc_id,
                component_id.clone()
            ),
        }
    }

    pub fn get_vrm_component_mut(&mut self, component_id: ComponentId) -> &mut Box<dyn VrmComponent + Send + 'static> {
        match self.vrm_components.get_mut(&component_id) {
            Some(container) => &mut container.vrm_component,
            None => panic!(
                "ErrorFailedToGetVrmComponentContainer: In the VrmComponentManager of ADC {}, was the ComponentId {} not found.",
                self.adc_id,
                component_id.clone()
            ),
        }
    }

    pub fn is_reservation_reserved(&self, reservation_id: ReservationId) -> bool {
        self.not_committed_reservations.contains_key(&reservation_id)
    }

    pub fn get_reserved_component(&self, reservation_id: ReservationId) -> Option<ComponentId> {
        self.not_committed_reservations.get(&reservation_id).cloned()
    }

    // --- Tracking Methods ---
    /// Registers a mapping for a single reservation (Atomic Job).
    pub fn register_allocation(&mut self, reservation_id: ReservationId, component_id: ComponentId) -> Option<ComponentId> {
        self.res_to_vrm_component.insert(reservation_id, component_id)
    }

    /// Merges a "transaction map" (from a Workflow Scheduler) into the global state.
    pub fn register_workflow_subtasks(&mut self, workflow_id: ReservationId, allocations: &HashMap<ReservationId, ComponentId>) {
        let subtask_ids: Vec<ReservationId> = allocations.keys().cloned().collect();

        // 1. Merge the allocation map (Who has what)
        self.res_to_vrm_component.extend(allocations.clone());

        // 2. Track relationship: Parent -> Children
        self.workflow_subtasks.insert(workflow_id.clone(), subtask_ids.clone());

        // 3. Track relationship: Child -> Parent
        for subtask_id in subtask_ids.clone() {
            self.reverse_workflow_subtasks.insert(subtask_id, workflow_id.clone());
        }

        // Check if reserve of all workflow subtask worked correctly
        for res_id in &subtask_ids {
            if !self.reservation_store.is_reservation_state_at_least(*res_id, ReservationState::ReserveAnswer) {
                panic!(
                    "ErrorVrmComponentManagerWorkflowSubtaskIsNotReserved: The registration of workflow {:?} for ADC {} failed, because workflow subtask {:?} was not successfully reserved (ReservationState is < ReserveAnswer). This suggests that there is an error during the reserve operation of the WorkflowScheduler or the VrmComponent reservation process.",
                    self.reservation_store.get_name_for_key(workflow_id),
                    self.adc_id,
                    self.reservation_store.get_name_for_key(*res_id)
                );
            }

            if !self.not_committed_reservations.contains_key(res_id) {
                panic!(
                    "ErrorVrmComponentManagerWorkflowSubtaskWasNotAddedToNotCommittedReservations: The registration of workflow {:?} for ADC {} failed, because workflow subtask {:?} was not successfully added to the not_committed_reservations. This suggests that there is an error during the reserve operation of the WorkflowScheduler or the VrmComponent reservation process.",
                    self.reservation_store.get_name_for_key(workflow_id),
                    self.adc_id,
                    self.reservation_store.get_name_for_key(*res_id)
                );
            }
        }
    }

    /// Retrieves the ComponentId responsible for a specific reservation.
    pub fn get_handler_id(&self, reservation_id: ReservationId) -> Option<ComponentId> {
        self.res_to_vrm_component.get(&reservation_id).cloned()
    }

    /// Retrieves the Parent Workflow ID for a given subtask.
    pub fn get_parent_workflow(&self, subtask_id: ReservationId) -> Option<ReservationId> {
        self.reverse_workflow_subtasks.get(&subtask_id).cloned()
    }

    /// Removes tracking for a reservation.
    /// If it's a workflow, this might need to clean up children, or children cleanup calls this.
    /// Currently, this removes the specific ID from the allocation map.
    pub fn remove_allocation(&mut self, reservation_id: &ReservationId) -> Option<ComponentId> {
        // Remove from reverse lookup if it exists
        self.reverse_workflow_subtasks.remove(reservation_id);
        // Remove from allocation map
        self.res_to_vrm_component.remove(reservation_id)
    }

    /// Removes all tracking associated with a workflow (children and the workflow entry itself).
    pub fn remove_workflow_tracking(&mut self, workflow_id: &ReservationId) {
        if let Some(subtasks) = self.workflow_subtasks.remove(workflow_id) {
            for subtask in subtasks {
                self.res_to_vrm_component.remove(&subtask);
                self.reverse_workflow_subtasks.remove(&subtask);
            }
        }
    }

    // Should aggregate hte router list of all components
    pub fn get_component_router_list(&self, component_id: ComponentId) -> Vec<RouterId> {
        self.vrm_components.get(&component_id).unwrap();
        todo!()
    }

    pub fn can_component_handel(&self, component_id: ComponentId, res: Reservation) -> bool {
        match self.vrm_components.get(&component_id) {
            Some(vrm_component) => vrm_component.vrm_component.can_handel(res),

            None => {
                log::debug!(
                    "NotFoundGridComponent: ADC {} requested can_handel request of reservation {}",
                    self.adc_id,
                    res.get_base_reservation().get_name()
                );
                return false;
            }
        }
    }

    // TODO Rename, if all use this function
    // Queues asks all child systems if they can handel request.
    // Returns true if one child system can handel request otherwise this function returns false.
    pub fn can_handel(&self, reservation_id: ReservationId) -> bool {
        for (_, container) in &self.vrm_components {
            if let Some(res) = self.reservation_store.get_reservation_snapshot(reservation_id) {
                if container.can_handel(res) {
                    return true;
                }
            } else {
                log::debug!(
                    "ReservationSnapShotFailed: ADC {} requested can_handel request of reservation {:?}",
                    self.adc_id,
                    self.reservation_store.get_name_for_key(reservation_id)
                );
            }
        }
        return false;
    }

    /// Get the total capacity of all connected VrmComponents
    pub fn get_total_capacity(&self) -> i64 {
        let mut total_capacity = 0;

        for (_, container) in &self.vrm_components {
            total_capacity += container.vrm_component.get_total_capacity()
        }

        total_capacity
    }

    /// Get the total link capacity of all connected VrmComponents
    pub fn get_total_link_capacity(&self) -> i64 {
        let mut total_link_capacity = 0;

        for (_, container) in &self.vrm_components {
            total_link_capacity += container.vrm_component.get_total_link_capacity()
        }

        total_link_capacity
    }

    /// Get the total node capacity of all connected VrmComponents
    pub fn get_total_node_capacity(&self) -> i64 {
        let mut total_node_capacity = 0;

        for (_, container) in &self.vrm_components {
            total_node_capacity += container.vrm_component.get_total_node_capacity()
        }

        total_node_capacity
    }

    /// Get the link resource_count of all connected VrmComponents
    pub fn get_link_resource_count(&self) -> usize {
        let mut link_resource_count = 0;

        for (_, container) in &self.vrm_components {
            link_resource_count += container.vrm_component.get_link_resource_count()
        }

        link_resource_count
    }

    /// Increments and returns the next available registration counter.
    pub fn get_new_registration_counter(&mut self) -> usize {
        let current = self.registration_counter;
        self.registration_counter += 1;
        return current;
    }

    /// Calculates the average link speed across all registered resources.
    pub fn get_average_link_speed(&self) -> f64 {
        if self.link_resource_count == 0 {
            return 0.0;
        }

        return self.total_link_capacity as f64 / self.link_resource_count as f64;
    }

    /// Registers a new **VrmComponent** with the manager.
    ///
    /// # Arguments
    /// * `vrm_component` - The `VrmComponent` instance to add.
    ///
    /// # Returns
    /// * `true` - If the VrmComponent was successfully added.
    /// * `false` - If the VrmComponent ID already exists or if an insertion error occurred (integrity compromised).
    pub fn add_vrm_component(
        &mut self,
        vrm_component: VrmComponentProxy,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
        number_of_real_slots: i64,
        slot_width: i64,
    ) -> bool {
        if self.vrm_components.contains_key(&vrm_component.get_id()) {
            log::error!(
                "Process of adding a new VrmComponent to the VrmComponentManger failed. It is not allowed to add the same VrmComponent multiple times. Please first delete the VrmComponent: {}.",
                vrm_component.get_id()
            );
            return false;
        }

        let vrm_component_id = vrm_component.get_id();
        let total_link_capacity = vrm_component.get_total_link_capacity();
        let link_resource_count = vrm_component.get_link_resource_count();
        let registration_index = self.get_new_registration_counter();

        let container = VrmComponentContainer::new(
            Box::new(vrm_component),
            simulator,
            reservation_store,
            registration_index,
            number_of_real_slots,
            slot_width,
            total_link_capacity,
            link_resource_count,
        );

        if self.vrm_components.insert(vrm_component_id.clone(), container).is_none() {
            return true;
        } else {
            log::error!(
                "Error happened in the process of adding a new VrmComponent: {} to the VrmComponentManager (Adc: {}). The VrmComponentManger is now compromised.",
                vrm_component_id,
                self.adc_id
            );
            return false;
        }
    }

    /// Removes an **VrmComponent** from the manager by its ID.
    ///
    /// Updates the total link capacity and link resource counts upon successful removal.
    ///
    /// # Arguments
    /// * `VrmComponentId` - The identifier of the VrmComponent to remove.
    ///
    /// # Returns
    /// * `true` - If the VrmComponent was found and removed.
    /// * `false` - If the VrmComponent ID was not found.
    pub fn delete_vrm_component(&mut self, component_id: ComponentId) -> bool {
        let container = self.vrm_components.remove(&component_id);
        match container {
            Some(container) => {
                self.total_link_capacity -= container.total_link_capacity;
                self.link_resource_count -= container.link_resource_count;
                // TODO
                // Also remove any reservations handled by this VrmComponent?
                // This would be complex as we'd need to iterate res_to_vrm_component
                return true;
            }
            None => {
                log::error!(
                    "The process of deleting the VrmComponent: {} form VrmComponentManager (Adc: {}). Failed, because the VrmComponentId was not present in the VrmComponentManager.",
                    component_id,
                    self.adc_id
                );
                return false;
            }
        }
    }

    /// Returns a list of all registered VrmComponent IDs in **random order**.
    ///
    /// # Returns
    /// A `Vec<VrmComponentId>` where the VrmComponentIds are in random order.
    pub fn get_random_ordered_vrm_components(&self) -> Vec<ComponentId> {
        let mut keys: Vec<ComponentId> = self.vrm_components.keys().cloned().into_iter().collect();
        keys.shuffle(&mut rng());
        return keys;
    }

    /// Returns a list of registered VrmComponent IDs sorted according to the specified strategy.
    /// If strict ordering is not required, `get_random_ordered_vrm_components` is preferred for performance.
    ///
    /// # Returns
    /// A `Vec<VrmComponentId>` sorted based on the comparator provided by `VrmComponentOrder`.
    pub fn get_ordered_vrm_components(&self, request_order: VrmComponentOrder) -> Vec<ComponentId> {
        let comparator = request_order.get_comparator();
        let mut components_vec: Vec<&VrmComponentContainer> = self.vrm_components.values().collect();

        components_vec.sort_unstable_by(|a, b| comparator(a, b));

        let sorted_keys: Vec<ComponentId> = components_vec.into_iter().map(|container| container.vrm_component.get_id()).collect();
        return sorted_keys;
    }

    /// Calculates the average **Satisfaction Score** (0.0 to 1.0) for the current schedule within a specific time window.
    /// This method queries all directly and indirectly connected AcIs and calculates the capacity-weighted average satisfaction.
    ///
    /// # Arguments
    /// * `start` - The start of the time window.
    /// * `end` - The end of the time window.
    /// * `shadow_schedule_id` - Optional ID. If provided, calculates based on the specified shadow schedule; otherwise uses the master schedule.
    ///
    /// # Returns
    /// A `f64` value between 0.0 (worst case) and 1.0 (best case). Returns 0.0 if total capacity is 0.
    pub fn get_satisfaction(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        log::debug!(
            "ADC: {} requests satisfaction of all AcIs with the ShadowScheduleId: {:?} the time window start: {} to end: {}",
            self.adc_id,
            shadow_schedule_id.clone(),
            start,
            end
        );

        let mut satisfaction_sum = 0.0;
        let mut total_capacity = 0.0;

        for (id, container) in self.vrm_components.iter_mut() {
            let satisfaction = container.vrm_component.get_satisfaction(start, end, shadow_schedule_id.clone());

            if satisfaction < 0.0 {
                log::debug!(
                    "Satisfaction of AcI is not allowed to be negative. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                let cap = container.vrm_component.get_total_node_capacity() as f64;
                satisfaction_sum += satisfaction * cap;
                total_capacity += cap;
            }
        }

        return if total_capacity > 0.0 { satisfaction_sum / total_capacity } else { 0.0 };
    }

    /// Calculates the system-wide **Satisfaction Score** (0.0 to 1.0) across the full range of every schedule.
    /// This method queries all directly and indirectly connected AcIs and calculates the capacity-weighted average.
    ///
    /// # Behavioral Note
    /// **Network AcIs:** This calculation generally excludes network AIs if their satisfaction/fragmentation
    /// functions are not implemented (returning -1). These are filtered out to prevent skewing the system metric.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - Optional ID. If provided, calculates based on the specified shadow schedule.
    ///                          (If None utilize master schedule)
    ///
    /// # Returns
    /// A `f64` value between 0.0 (worst case) and 1.0 (best case).
    pub fn get_system_satisfaction(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        log::debug!("ADC: {} requests system satisfaction of all AcIs with the ShadowScheduleId: {:?}.", self.adc_id, shadow_schedule_id.clone());

        let mut satisfaction_sum = 0.0;
        let mut total_capacity = 0.0;

        for (id, container) in self.vrm_components.iter_mut() {
            let satisfaction = container.vrm_component.get_system_satisfaction(shadow_schedule_id.clone());
            if satisfaction < 0.0 {
                log::debug!(
                    "System satisfaction of AcI is not allowed to be negative. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                let cap = container.vrm_component.get_total_node_capacity() as f64;
                satisfaction_sum += satisfaction * cap;
                total_capacity += cap;
            }
        }

        return if total_capacity > 0.0 { satisfaction_sum / total_capacity } else { 0.0 };
    }

    /// Computes the **Load Metric** for a specific time range.
    /// This method aggregates the load of all directly and indirectly connected AcIs.
    /// **Note:** Only jobs submitted via this ADC are typically counted; actual load on the physical resource
    /// may be higher due to local jobs or other ADCs.
    ///
    /// # Arguments
    /// * `start` - Start of the analysis window in seconds (VRM Time).
    /// * `end` - End of the analysis window in seconds (VRM Time).
    /// * `shadow_schedule_id` - Optional ID for shadow schedule analysis (If None utilize master schedule).
    ///
    /// # Returns
    /// A `LoadMetric` struct containing utilization, start/end times, and capacity details.
    pub fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        let mut total_possible_reserved_capacity = 0.0;
        let mut total_average_reserved_capacity = 0.0;
        let mut earliest_start = i64::MAX;
        let mut latest_end = i64::MIN;
        let mut num_of_valid_components = 0;

        for (id, container) in self.vrm_components.iter() {
            let load_matic = container.vrm_component.get_load_metric(start, end, shadow_schedule_id.clone());

            if load_matic.start_time < 0 {
                log::debug!(
                    "Get Load Metric with negative start time is not allowed. ADC: {}, child VrmComponent:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                total_average_reserved_capacity += load_matic.avg_reserved_capacity;
                total_possible_reserved_capacity += load_matic.possible_capacity;
                num_of_valid_components += 1;

                if earliest_start > load_matic.start_time {
                    earliest_start = load_matic.start_time;
                }

                if latest_end < load_matic.end_time {
                    latest_end = load_matic.end_time;
                }
            }
        }

        let mut utilization: f64 = 0.0;
        if total_possible_reserved_capacity > 0.0 {
            utilization = total_average_reserved_capacity / total_possible_reserved_capacity;
        }

        if num_of_valid_components > 0 {
            return LoadMetric::new(
                earliest_start,
                latest_end,
                total_average_reserved_capacity / num_of_valid_components as f64,
                total_possible_reserved_capacity / num_of_valid_components as f64,
                utilization,
            );
        } else {
            return LoadMetric::new(earliest_start, latest_end, 0.0, 0.0, utilization);
        }
    }

    /// Computes the **Load Metric** for the entire simulation timeline.
    /// Aggregates metrics from all valid AcIs to provide a high-level view of system utilization.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - Optional ID for shadow schedule analysis (If None utilize master schedule).
    ///
    /// # Returns
    /// A `LoadMetric` representing the average reserved capacity and utilization across the simulation.
    pub fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        let mut total_possible_reserved_capacity = 0.0;
        let mut total_average_reserved_capacity = 0.0;
        let mut earliest_start = i64::MAX;
        let mut latest_end = i64::MIN;
        let mut num_of_valid_components = 0;

        for (id, container) in self.vrm_components.iter_mut() {
            let load_matic = container.vrm_component.get_simulation_load_metric(shadow_schedule_id.clone());

            if load_matic.start_time < 0 {
                log::debug!(
                    "Get Load Metric with negative start time is not allowed. ADC: {}, child VrmComponent:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                total_average_reserved_capacity += load_matic.avg_reserved_capacity;
                total_possible_reserved_capacity += load_matic.possible_capacity;
                num_of_valid_components += 1;

                if earliest_start > load_matic.start_time {
                    earliest_start = load_matic.start_time;
                }

                if latest_end < load_matic.end_time {
                    latest_end = load_matic.end_time;
                }
            }
        }

        let mut utilization: f64 = 0.0;
        if total_possible_reserved_capacity > 0.0 {
            utilization = total_average_reserved_capacity / total_possible_reserved_capacity;
        }

        if num_of_valid_components > 0 {
            return LoadMetric::new(
                earliest_start,
                latest_end,
                total_average_reserved_capacity / num_of_valid_components as f64,
                total_possible_reserved_capacity / num_of_valid_components as f64,
                utilization,
            );
        } else {
            return LoadMetric::new(earliest_start, latest_end, 0.0, 0.0, utilization);
        }
    }

    pub fn probe(
        &mut self,
        component_id: ComponentId,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> ProbeReservations {
        match self.vrm_components.get_mut(&component_id) {
            Some(container) => container.vrm_component.probe(reservation_id, shadow_schedule_id),
            None => {
                log::error!(
                    "ComponentManagerHasNotFoundGridComponent: ComponentManager of ADC {}, requested component {} for probe request of reservation {:?} on shadow_schedule {:?}",
                    self.adc_id,
                    component_id,
                    reservation_id,
                    shadow_schedule_id
                );

                return ProbeReservations::new(reservation_id, self.reservation_store.clone());
            }
        }
    }

    pub fn probe_all_components(&mut self, reservation_id: ReservationId) -> ProbeReservations {
        let mut probe_results = ProbeReservations::new(reservation_id, self.reservation_store.clone());

        for (_, container) in &mut self.vrm_components {
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if container.can_handel(res_snapshot) {
                let mut probe_reservations = container.vrm_component.probe(reservation_id, None);

                // Do not trust answer of lower GridComponent
                // Validation of probe answers
                for probe_res_id in probe_reservations.get_ids() {
                    if self.reservation_store.get_assigned_start(probe_res_id) < self.reservation_store.get_booking_interval_start(probe_res_id)
                        || self.reservation_store.get_assigned_end(probe_res_id) > self.reservation_store.get_booking_interval_end(probe_res_id)
                    {
                        probe_reservations.delete_reservation(probe_res_id);
                        log::error!("Invalid Answer.");
                    }
                }

                probe_results.add_probe_reservations(probe_reservations);
            }
        }

        if probe_results.is_empty() {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
        }
        return probe_results;
    }

    pub fn reserve(
        &mut self,
        component_id: ComponentId,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> ReservationId {
        match self.vrm_components.get_mut(&component_id) {
            Some(container) => {
                container.vrm_component.reserve(reservation_id, shadow_schedule_id);

                if self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                    self.not_committed_reservations.insert(reservation_id, component_id);
                }

                return reservation_id;
            }
            None => {
                log::error!(
                    "ComponentManagerHasNotFoundGridComponent: ComponentManager of ADC {}, requested component {} for reserve request of reservation {:?} on shadow_schedule {:?}",
                    self.adc_id,
                    component_id,
                    reservation_id,
                    shadow_schedule_id
                );

                return reservation_id;
            }
        }
    }

    pub fn reserve_without_check(&mut self, component_id: ComponentId, reservation_id: ReservationId) {
        match self.vrm_components.get_mut(&component_id) {
            Some(container) => container.schedule.reserve_without_check(reservation_id),
            None => {
                log::error!(
                    "ComponentManagerHasNotFoundGridComponent: ComponentManager of ADC {}, requested component {} for reserve_without_check request of reservation {:?} on schedule",
                    self.adc_id,
                    component_id,
                    reservation_id,
                );
            }
        }
    }
}

impl VrmComponentManager {
    pub fn log_stat(&mut self, command: String, reservation_id: ReservationId, arrival_time_at_aci: i64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let processing_time = self.simulator.get_current_time_in_ms() - arrival_time_at_aci;

        if let Some(res_handle) = self.reservation_store.get(reservation_id) {
            let (start, end, res_name, capacity, workload, state, proceeding, num_tasks) = {
                let res = res_handle.read().unwrap();

                let start = res.get_base_reservation().get_assigned_start();
                let end = res.get_base_reservation().get_assigned_end();
                let name = res.get_base_reservation().get_name().clone();
                let cap = res.get_base_reservation().get_reserved_capacity();
                let workload = res.get_base_reservation().get_task_duration() * cap;
                let state = res.get_base_reservation().get_state();
                let proceeding = res.get_base_reservation().get_reservation_proceeding();

                // TODO Java implementation also proceeded workflows if so, num_task should not be always be 1 (implement get_task_count())
                let tasks = 42;

                (start, end, name, cap, workload, state, proceeding, tasks)
            };

            let load_metric = self.get_load_metric(start, end, None);

            tracing::info!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished",
                ComponentType = %self.adc_id.clone(),
                ComponentUtilization = load_metric.utilization,
                ComponentCapacity = load_metric.possible_capacity,
                ComponentFragmentation = self.get_system_satisfaction(None),
                ReservationName = %res_name,
                ReservationCapacity = capacity,
                ReservationWorkload = workload,
                ReservationState = ?state,
                ReservationProceeding = ?proceeding,
                NumberOfTasks = num_tasks,
                Command = command,
                ProcessingTime = processing_time,
            );
        } else {
            // Handling in case reservation is missing (e.g. deleted/cleaned up)

            tracing::warn!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished (Reservation Missing/Deleted)",
                ComponentType = %self.adc_id,
                ReservationId = ?reservation_id,
                Command = command,
                ProcessingTime = processing_time,
            );
        }
    }
}
