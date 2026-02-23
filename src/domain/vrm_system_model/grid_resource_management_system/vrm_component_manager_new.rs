use std::{cmp::Ordering, collections::HashMap};

use crate::domain::vrm_system_model::{
    grid_resource_management_system::{
        order_res_vrm_component_database::OrderResVrmComponentDatabase,
        vrm_component_manager::{DUMMY_COMPONENT_ID, VrmComponentManager},
        vrm_component_order::VrmComponentOrder,
    },
    reservation::{
        probe_reservations::{ProbeReservationComparator, ProbeReservations},
        reservation::ReservationState,
        reservation_store::ReservationId,
    },
    utils::id::{ComponentId, ShadowScheduleId},
};

impl VrmComponentManager {
    /// Performs the commit operation at the specific underlying component.
    ///
    /// This is used internally for both atomic tasks and sub-tasks within a workflow.
    /// If the component is a dummy/internal component, the state is updated locally.
    /// Returns `true` if the component successfully committed the reservation.
    pub fn commit_at_component(&mut self, reservation_id: ReservationId, component_id: ComponentId) -> bool {
        // Is dummy task/ "Internal task"
        if component_id == *DUMMY_COMPONENT_ID {
            self.reservation_store.update_state(reservation_id, ReservationState::Committed);
            return true;
        }

        let container = self.get_vrm_component_container_mut(component_id.clone());
        if container.vrm_component.commit(reservation_id) {
            self.update_commit_tracking(reservation_id, component_id);
            return true;
        }

        // If commit fails, clean up local schedule and global mapping
        container.schedule.delete_reservation(reservation_id);
        self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
        return false;
    }

    pub fn update_commit_tracking(&mut self, reservation_id: ReservationId, component_id: ComponentId) {
        if !self.is_reservation_reserved(reservation_id) {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            log::error!(
                "ErrorInCommitPreProcess: Commit at Component {} of ADC {} failed for Reservation {:?}. There was no reserve at a 
                    VrmComponent for the reservation found. Should happen before.",
                component_id,
                self.adc_id,
                self.reservation_store.get_name_for_key(reservation_id)
            );
        }
        if self.committed_reservations.contains_key(&reservation_id) {
            log::error!(
                "ErrorInCommitPreProcess: Commit at Component {} of ADC {} failed for Reservation {:?}. The reservation was already committed to a VrmComponent",
                component_id,
                self.adc_id,
                self.reservation_store.get_name_for_key(reservation_id)
            );
        }

        self.not_committed_reservations.remove(&reservation_id);
        self.committed_reservations.insert(reservation_id, component_id);
    }

    /// Transitions all committed reservations into state `ReservationState::Rejected` state following a scheduling or resource failure.
    pub fn handle_commit_failure(&mut self, clean_vrm_of_res_ids: Vec<ReservationId>) {
        for reservation_id in &clean_vrm_of_res_ids {
            self.reservation_store.update_state(*reservation_id, ReservationState::Rejected);
            if !self.delete_task_at_component(*reservation_id, None) {
                panic!("Deletion of Committed task failed.");
            }
        }
    }

    /// Creates a new Shadow Schedule environment.
    ///
    /// This snapshots the current ReservationStore and Component Mappings and propagates the creation
    /// to all child components.
    pub fn create_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if self.shadow_schedule_reservations.contains_key(&shadow_schedule_id) {
            log::error!(
                "VrmComponentManagerShadowScheduleWithIdExistsAlready: The process of creating a new shadow Schedule for the ADC {} with ShadowScheduleId {:?} failed, because the provided ShadowScheduleId already exists, please first delete the other ShadowScheduleId.",
                self.adc_id,
                shadow_schedule_id
            );
            return false;
        }

        // 1. Snapshot the local state (ReservationStore and Allocation Map)
        let shadow_store = self.reservation_store.snapshot();
        // We clone the current allocation map (Who handles what) to serve as the baseline for the shadow schedule
        let shadow_map = self.res_to_vrm_component.clone();

        // 2. Propagate creation to all children (VrmComponents)
        for container in self.vrm_components.values_mut() {
            if !container.vrm_component.create_shadow_schedule(shadow_schedule_id.clone()) {
                log::error!("Failed to create shadow schedule on child component {:?}", container.vrm_component.get_id());
                // In a robust system, we would trigger a rollback here
                return false;
            }
        }

        // 3. Store the shadow context
        self.shadow_schedule_reservations.insert(shadow_schedule_id, (shadow_map, shadow_store));

        return true;
    }

    /// Discards a Shadow Schedule without applying changes (Rollback).
    pub fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if !self.shadow_schedule_reservations.contains_key(&shadow_schedule_id) {
            return false;
        }

        // 1. Propagate deletion to all children
        for container in self.vrm_components.values_mut() {
            container.vrm_component.delete_shadow_schedule(shadow_schedule_id.clone());
        }

        // 2. Remove local shadow context
        self.shadow_schedule_reservations.remove(&shadow_schedule_id);

        return true;
    }

    /// Commits the Shadow Schedule to be the new Master Schedule.
    ///
    /// This replaces the live state with the shadow state.
    pub fn commit_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if !self.shadow_schedule_reservations.contains_key(&shadow_schedule_id) {
            log::error!("Cannot commit shadow schedule {:?} as it does not exist.", shadow_schedule_id);
            return false;
        }

        // 1. Propagate commit to all children first
        for container in self.vrm_components.values_mut() {
            if !container.vrm_component.commit_shadow_schedule(shadow_schedule_id.clone()) {
                log::error!("Child component {:?} failed to commit shadow schedule.", container.vrm_component.get_id());
                return false;
            }
        }

        // 2. Atomic Switch: Replace Master State with Shadow State
        let (shadow_map, shadow_store) = self.shadow_schedule_reservations.remove(&shadow_schedule_id).unwrap();

        // Update the component mapping (Who handles what)
        self.res_to_vrm_component = shadow_map;

        // Update the reservation store (The source of truth for reservation states)
        self.reservation_store = shadow_store;

        // Rebuild derived mappings (committed/not_committed) based on the new store state
        // This ensures internal consistency after the swap
        self.committed_reservations.clear();
        self.not_committed_reservations.clear();

        for (res_id, component_id) in &self.res_to_vrm_component {
            let state = self.reservation_store.get_state(*res_id);
            match state {
                ReservationState::Committed => {
                    self.committed_reservations.insert(*res_id, component_id.clone());
                }
                ReservationState::ReserveAnswer => {
                    self.not_committed_reservations.insert(*res_id, component_id.clone());
                }
                _ => {} // Ignore others or log warning
            }
        }

        return true;
    }

    /// Probes all available VrmComponents and selects the best candidate based on the provided comparison function.
    ///
    /// This implements a "Best Fit" strategy, useful for optimizing resource utilization or
    /// meeting Earliest Finish Time (EFT) constraints.
    pub fn reserve_task_at_best_vrm_component<F>(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
        probe_reservation_comparator: ProbeReservationComparator,
        reservation_order: F,
    ) -> Option<ReservationId>
    where
        F: Fn(ReservationId, ReservationId) -> Ordering + 'static,
    {
        let try_n_probe_reservations = 5;
        let mut probe_reservations = ProbeReservations::new(reservation_id, self.reservation_store.clone());

        for component_id in self.get_random_ordered_vrm_components() {
            // Get Reservation Clone of the ShadowScheduleId or MasterSchedule
            let res_snapshot = if let Some(sid) = &shadow_schedule_id {
                if let Some((_, store)) = self.shadow_schedule_reservations.get(sid) {
                    store.get_reservation_snapshot(reservation_id)
                } else {
                    self.reservation_store.get_reservation_snapshot(reservation_id)
                }
            } else {
                self.reservation_store.get_reservation_snapshot(reservation_id)
            };

            if let Some(res) = res_snapshot {
                if self.can_component_handel(component_id.clone(), res) {
                    probe_reservations
                        .add_probe_reservations(self.get_vrm_component_mut(component_id.clone()).probe(reservation_id, shadow_schedule_id.clone()));
                }
            }
        }

        for _ in 0..=try_n_probe_reservations {
            if probe_reservations.prompt_best(reservation_id, probe_reservation_comparator.clone()) {
                // 1. Prepare the gate
                let gate = self.sync_registry.create_gate(reservation_id);

                // 2. Trigger the AcI by updating the store
                self.reservation_store.update_state(reservation_id, ReservationState::ReserveProbeReservation);

                // TODO Add parameter to a config
                // 3. BLOCK here. This thread sleeps until AcI calls notify().
                let result = gate.wait_with_timeout(std::time::Duration::from_secs(15));

                // 4. Clean up the registry
                self.sync_registry.remove_gate(reservation_id);

                // Check if update of local schedule was successful
                if result.state == ReservationState::ReserveAnswer {
                    log::info!("Reservation {:?} successful!", reservation_id);

                    // Register new schedule Sub-Task
                    // Update grid_component_res_database for rollback and for ADC to keep track
                    // Update Transaction Log
                    if grid_component_res_database.contains_key(&reservation_id) {
                        log::error!(
                            "ErrorReservationWasReservedInMultipleGridComponents: The reservation {:?} was multiple times to the GirdComponent {} submitted.",
                            self.reservation_store.get_name_for_key(reservation_id),
                            result.aci_id.clone().unwrap(),
                        );
                    }
                    grid_component_res_database.insert(reservation_id, result.aci_id.unwrap());
                    return Some(reservation_id);
                }
            }
        }
        return None;
    }

    /// Submits a task to the first VrmComponent that accepts the reservation based on the defined `VrmComponentOrder`.
    pub fn reserve_task_at_first_grid_component(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        vrm_component_order: VrmComponentOrder,
    ) -> ReservationId {
        // Wrong order
        for component_id in self.get_ordered_vrm_components(vrm_component_order) {
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.can_component_handel(component_id.clone(), res_snapshot) {
                let reserve_res_id = self.reserve(component_id.clone(), reservation_id, shadow_schedule_id.clone());

                let is_reserved = if let Some(sid) = &shadow_schedule_id {
                    if let Some((_, store)) = self.shadow_schedule_reservations.get(sid) {
                        store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer)
                    } else {
                        false
                    }
                } else {
                    self.reservation_store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer)
                };

                if is_reserved {
                    self.update_reserve_tracking(reserve_res_id, component_id.clone(), shadow_schedule_id);

                    // Update VrmComponent's local view (schedule) of the underlying VrmComponents
                    self.reserve_without_check(component_id.clone(), reserve_res_id);
                    return reserve_res_id;
                }
            }
        }

        // Update failure state in appropriate store
        if let Some(sid) = &shadow_schedule_id {
            if let Some((_, store)) = self.shadow_schedule_reservations.get(sid) {
                store.update_state(reservation_id, ReservationState::Rejected);
            }
        } else {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
        }

        return reservation_id;
    }

    //TODO ShadowSchedule is it right, do to tho this with the reservation_store of the shadow_schedule?
    pub fn update_reserve_tracking(
        &mut self,
        reservation_id: ReservationId,
        component_id: ComponentId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) {
        if shadow_schedule_id.is_none() {
            let old_value = self.not_committed_reservations.insert(reservation_id, component_id.clone());

            if !old_value.is_none() {
                panic!(
                    "ErrorVrmManagerDuplicateReserveReservationInNotCommittedReservations: The tracking update of a reserved reservation of ADC {} failed. The Reservation {:?} was already reserved before on VrmComponent {}. The new reserve was performed for VrmComponent {}",
                    self.adc_id,
                    self.reservation_store.get_name_for_key(reservation_id),
                    old_value.unwrap(),
                    component_id
                );
            }

            if !self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                panic!(
                    "ErrorVrmManagerReservationStateIsNotAtLeastReserveAnswer: The tracking update of a reserved reservation of ADC {} failed. The Reservation {:?} reserved on VrmComponent {} must have at least ReservationState::ReservedAnswer, but as only state {:?}.",
                    self.adc_id,
                    self.reservation_store.get_name_for_key(reservation_id),
                    component_id,
                    self.reservation_store.get_state(reservation_id)
                );
            }

            let old_value = self.register_allocation(reservation_id, component_id.clone());

            if !old_value.is_none() {
                panic!(
                    "ErrorVrmManagerDuplicateReserveInResToVrmComponent: The tracking update of a reserved reservation of ADC {} failed. The Reservation {:?} was already reserved before on VrmComponent {}. The new reserve was performed for VrmComponent {}",
                    self.adc_id,
                    self.reservation_store.get_name_for_key(reservation_id),
                    old_value.unwrap(),
                    component_id
                );
            }
        } else {
            let (shadow_not_committed_reservations, shadow_reservation_store) =
                self.shadow_schedule_reservations.get_mut(&shadow_schedule_id.clone().unwrap()).expect("ErrorVrmManagerShadowScheduleWasNotFound");

            // TODO: In shadow mode, we often use `res_to_vrm_component` (shadow_map) to track everything.
            // `not_committed_reservations` is technically derived from state.
            // Here we update the map directly.

            let old_value = shadow_not_committed_reservations.insert(reservation_id, component_id.clone());

            if !old_value.is_none() {
                panic!(
                    "ErrorVrmManagerDuplicateReserve: The reservation tracking update of a reserved reservation of ADC {} on ShadowSchedule {:?} failed. The Reservation {:?} was already reserved before on VrmComponent {}. The new reserve was performed for VrmComponent {}",
                    self.adc_id,
                    shadow_schedule_id,
                    shadow_reservation_store.get_name_for_key(reservation_id),
                    old_value.unwrap(),
                    component_id
                );
            }

            if !shadow_reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                panic!(
                    "ErrorVrmManagerReservationStateIsNotAtLeastReserveAnswer: The tracking update of a reserved reservation of ADC {} on ShadowSchedule {:?} failed. The Reservation {:?} reserved on VrmComponent {} must have at least ReservationState::ReservedAnswer, but as only state {:?}.",
                    self.adc_id,
                    shadow_schedule_id,
                    shadow_reservation_store.get_name_for_key(reservation_id),
                    component_id,
                    shadow_reservation_store.get_state(reservation_id)
                );
            }
        }
    }

    pub fn delete_task_at_component(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> bool {
        let mut target_component = None;

        if let Some(sid) = &shadow_schedule_id {
            if let Some((shadow_map, _)) = self.shadow_schedule_reservations.get(&sid) {
                target_component = shadow_map.get(&reservation_id).cloned();
            }
        } else {
            target_component = self.res_to_vrm_component.get(&reservation_id).cloned();
        }

        match target_component {
            Some(component_id) => {
                // No Real Task
                if component_id == *DUMMY_COMPONENT_ID {
                    if let Some(sid) = &shadow_schedule_id {
                        if let Some((_, store)) = self.shadow_schedule_reservations.get(&sid) {
                            store.update_state(reservation_id, ReservationState::Deleted);
                        }
                    } else {
                        self.reservation_store.update_state(reservation_id, ReservationState::Deleted);
                    }
                    return true;
                }

                let container = self.get_vrm_component_container_mut(component_id.clone());

                container.vrm_component.delete(reservation_id, shadow_schedule_id.clone());

                // Note: We check the store to verify deletion.
                // If shadow, we check the shadow store
                let is_deleted = if let Some(sid) = &shadow_schedule_id {
                    // Check shadow store state
                    if let Some((_, store)) = self.shadow_schedule_reservations.get(&sid) {
                        store.get_state(reservation_id) == ReservationState::Deleted
                    } else {
                        false
                    }
                } else {
                    self.reservation_store.get_state(reservation_id) == ReservationState::Deleted
                };

                if is_deleted {
                    // Update Local view
                    let container = self.get_vrm_component_container_mut(component_id);
                    container.schedule.delete_reservation(reservation_id);

                    // Cleanup Mapping
                    if let Some(sid) = &shadow_schedule_id {
                        if let Some((shadow_map, _)) = self.shadow_schedule_reservations.get_mut(&sid) {
                            shadow_map.remove(&reservation_id);
                        }
                    } else {
                        self.res_to_vrm_component.remove(&reservation_id);
                    }

                    return true;
                }

                self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                return false;
            }
            None => {
                log::error!(
                    "ReservationForDeletionWasNotFound: In ADC {} ShadowSchedule {:?} was Reservation {:?} not found.",
                    self.adc_id,
                    shadow_schedule_id,
                    self.reservation_store.get_name_for_key(reservation_id)
                );
                return false;
            }
        }
    }
}
