use std::{cmp::Ordering, collections::HashMap};

use crate::domain::vrm_system_model::{
    grid_resource_management_system::{
        order_res_vrm_component_database::OrderResVrmComponentDatabase,
        vrm_component_manager::{DUMMY_COMPONENT_ID, VrmComponentManager},
        vrm_component_order::VrmComponentOrder,
    },
    reservation::{reservation::ReservationState, reservation_store::ReservationId},
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
            if !self.delete_task_at_component(reservation_id.clone(), self.res_to_vrm_component.get(reservation_id).unwrap().clone(), None) {
                panic!("Deletion of Committed task failed.");
            }
        }
    }

    pub fn create_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if self.shadow_schedule_reservations.contains_key(&shadow_schedule_id) {
            log::error!("VrmComponentManagerShadowScheduleWithIdExistsAlready: The process of creating a new shadow Schedule for the ADC {} with ShadowScheduleId {:?} failed, because the provided ShadowScheduleId already exists, please first delete the other ShadowScheduleId.", self.adc_id, shadow_schedule_id);   
            return false; 
        }

        self.shadow_schedule_reservations.insert(shadow_schedule_id, (HashMap::new(), self.reservation_store.snapshot()));
        
        todo!()
    }

    pub fn rollback_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    pub fn commit_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        // TODO Add ReservationStore Listener
        todo!()
    }

    pub fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    /// Probes all available VrmComponents and selects the best candidate based on the provided comparison function.
    ///
    /// This implements a "Best Fit" strategy, useful for optimizing resource utilization or
    /// meeting Earliest Finish Time (EFT) constraints.
    /// TODO should be moved to VrmComponentManager
    pub fn reserve_task_at_best_vrm_component<F>(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        grid_component_res_database: &mut HashMap<ReservationId, ComponentId>,
        vrm_component_order: VrmComponentOrder,
        reservation_order: F,
    ) -> Option<ReservationId>
    where
        F: Fn(ReservationId, ReservationId) -> Ordering + 'static,
    {
        let mut order_grid_component_res_database = OrderResVrmComponentDatabase::new(reservation_order, vrm_component_order.get_comparator());

        for component_id in self.get_random_ordered_vrm_components() {
            let res_snapshot = self.reservation_store.get_reservation_snapshot(reservation_id).unwrap();

            if self.can_component_handel(component_id.clone(), res_snapshot) {
                let probe_reservations = self.get_vrm_component_mut(component_id.clone()).probe(reservation_id, shadow_schedule_id.clone());

                // Do not trust answer of lower GridComponent
                // Validation of probe answers
                for prob_reservation_id in probe_reservations.get_ids() {
                    if self.reservation_store.get_assigned_start(prob_reservation_id)
                        < self.reservation_store.get_booking_interval_start(prob_reservation_id)
                        || self.reservation_store.get_assigned_end(prob_reservation_id)
                            > self.reservation_store.get_booking_interval_end(prob_reservation_id)
                    {
                        log::error!("Invalid Answer.");
                    }
                }

                order_grid_component_res_database.put_all(probe_reservations);
            }
        }

        // Choose reservation candidate with EFT and reserve it
        for reservation_id in order_grid_component_res_database.sorted_key_set(&self) {
            let component_id = order_grid_component_res_database.store.get(&reservation_id).unwrap();

            let candidate_id = self.reserve(component_id.clone(), reservation_id, None);

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
                self.reserve_without_check(component_id.clone(), candidate_id);

                if self.reservation_store.is_reservation_state_at_least(candidate_id, ReservationState::ReserveAnswer) {
                    log::error!("Reserve of reservation {:?} in local schedule of GridComponent {:?} failed.", candidate_id, component_id);
                }
                return Some(candidate_id);
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

                if self.reservation_store.is_reservation_state_at_least(reserve_res_id, ReservationState::ReserveAnswer) {
                    self.update_reserve_tracking(reserve_res_id, component_id.clone(), shadow_schedule_id);

                    // Update VrmComponent's local view (schedule) of the underlying VrmComponents
                    self.reserve_without_check(component_id.clone(), reserve_res_id);
                    return reserve_res_id;
                }
            }
        }

        self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
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

            if !self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                panic!(
                    "ErrorVrmManagerReservationStateIsNotAtLeastReserveAnswer: The tracking update of a reserved reservation of ADC {} on ShadowSchedule {:?} failed. The Reservation {:?} reserved on VrmComponent {} must have at least ReservationState::ReservedAnswer, but as only state {:?}.",
                    self.adc_id,
                    shadow_schedule_id,
                    shadow_reservation_store.get_name_for_key(reservation_id),
                    component_id,
                    self.reservation_store.get_state(reservation_id)
                );
            }

            // TODO Add Reservation to res_to_vrm_component
        }
    }

    pub fn delete_task_at_component(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> bool {
        let mut component_id; 

        if shadow_schedule_id.is_none() {
            let (res_id_to_component_id, shadow_reservation_store) = self.shadow_schedule_reservations.get_mut(&shadow_schedule_id.unwrap()).unwrap();
        }


        match self.res_to_vrm_component.remove(&reservation_id) {
            Some(component_id) => {
                // No Real Task
                if component_id == *DUMMY_COMPONENT_ID {
                    self.reservation_store.update_state(reservation_id, ReservationState::Deleted);
                    return true;
                }

                let mut container = self.get_vrm_component_container_mut(component_id);

                container.vrm_component.delete(reservation_id, shadow_schedule_id)

                if self.reservation_store.get_state(reservation_id) == ReservationState::Deleted {
                    // Update Local view 
                    container.schedule.delete_reservation(reservation_id);
                    // TODO Update Globe View 
                    return true;
                }

                self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                return false;

            }
            None => {
                log::error!("ReservationForDeletionWasNotFound: In ADC {} ShadowSchedule {:?} was Reservation {:?} not found.", self.adc_id, shadow_schedule_id, self.reservation_store.get_name_for_key(reservation_id));
                return false;
            }
        }
        if self.reservation
    }
}
