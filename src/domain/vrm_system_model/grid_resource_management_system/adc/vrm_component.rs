use std::collections::HashMap;

use crate::domain::vrm_system_model::{
    grid_resource_management_system::{adc::ADC, vrm_component_trait::VrmComponent},
    reservation::{
        probe_reservations::ProbeReservations,
        reservation::{Reservation, ReservationState},
        reservation_store::ReservationId,
    },
    utils::{
        id::{ComponentId, RouterId, ShadowScheduleId},
        load_buffer::LoadMetric,
    },
};

impl VrmComponent for ADC {
    fn get_id(&self) -> ComponentId {
        ComponentId::new(self.id.to_string())
    }

    fn get_total_capacity(&self) -> i64 {
        self.manager.get_total_capacity()
    }

    fn get_total_link_capacity(&self) -> i64 {
        self.manager.get_total_link_capacity()
    }

    fn get_total_node_capacity(&self) -> i64 {
        self.manager.get_total_node_capacity()
    }

    fn get_link_resource_count(&self) -> usize {
        self.manager.get_link_resource_count()
    }

    fn can_handel(&self, res: Reservation) -> bool {
        for component_id in self.manager.get_random_ordered_vrm_components() {
            if self.manager.can_component_handel(component_id, res.clone()) {
                return true;
            }
        }
        false
    }

    fn commit(&mut self, reservation_id: ReservationId) -> bool {
        let arrival_time = self.simulator.get_current_time_in_ms();
        log::info!("ADC {} commits reservation {:?}.", self.id, self.reservation_store.get_name_for_key(reservation_id));

        // Get ComponentId where Reservation is reserved
        // Most like likely happen before if not reserve now.
        if !self.manager.is_reservation_reserved(reservation_id) {
            log::info!(
                "There was no reserve performed for the commit of reservation {:?}, try to reserve reservation now.",
                self.reservation_store.get_name_for_key(reservation_id)
            );

            // Can VrmManagerHandel request
            if !self.manager.can_handel(reservation_id) {
                self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                log::debug!(
                    "Commit at ADC {} failed of Reservation {:?} was rejected, because VrmComponents can not handel reservation and no reservation was done prior.",
                    self.id,
                    self.reservation_store.get_name_for_key(reservation_id)
                );
                self.log_stat("Commit".to_string(), reservation_id, arrival_time);
                return false;
            }

            // Reserve now reservation
            self.reserve(reservation_id, None);

            if !self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                log::debug!(
                    "Commit at ADC {} failed of Reservation {:?} was rejected, because VrmComponents can not handel reservation and no reservation was done prior.",
                    self.id,
                    self.reservation_store.get_name_for_key(reservation_id)
                );

                self.log_stat("Commit".to_string(), reservation_id, arrival_time);
                return false;
            }
        }

        // Get ComponentId where Reservation was reserved
        let component_id = if self.manager.is_reservation_reserved(reservation_id) {
            self.manager.get_reserved_component(reservation_id).unwrap()
        } else {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            log::error!(
                "ErrorInReservationProcess: Commit at ADC {} failed of Reservation {:?}. There was no reserve at a 
                    VrmComponent for the reservation found. Should happen before.",
                self.id,
                self.reservation_store.get_name_for_key(reservation_id)
            );
            return false;
        };

        // Perform Commit at VrmComponentManager (Single or Workflow Reservation?)
        if self.reservation_store.is_workflow(reservation_id) {
            let sub_ids = self.workflow_scheduler.as_mut().unwrap().get_sub_ids(reservation_id);

            for sub_res_id in sub_ids.clone() {
                let component_answer = self.manager.commit_at_component(sub_res_id, component_id.clone());
                let state = self.reservation_store.get_state(sub_res_id);

                // Check if this specific sub-component succeeded
                if state != ReservationState::Committed || !component_answer {
                    log::error!("Sub-task {:?} failed in workflow {:?}", sub_res_id, reservation_id);
                    let mut clean_vrm_of_res_ids = sub_ids.clone();
                    clean_vrm_of_res_ids.push(reservation_id);

                    self.manager.handle_commit_failure(clean_vrm_of_res_ids);
                    return false;
                }
            }

            self.workflow_scheduler.as_mut().unwrap().finalize_commit(reservation_id);
        } else {
            // Non-workflow atomic job
            let is_committed = self.manager.commit_at_component(reservation_id, component_id);
            if !is_committed {
                return false;
            }
        }

        log::debug!("Committed at ADC {} Reservation {:?}.", self.id, self.reservation_store.get_name_for_key(reservation_id));

        self.log_stat("Commit".to_string(), reservation_id, arrival_time);
        return true;
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        self.manager.commit_shadow_schedule(shadow_schedule_id)
    }

    fn create_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        self.manager.create_shadow_schedule(shadow_schedule_id)
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        todo!()
    }

    fn delete(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ReservationId {
        let arrival_time = self.simulator.get_current_time_in_ms();
        log::info!("ADC Delete: Delete on ADC {} the Reservation {:?}", self.id, self.reservation_store.get_name_for_key(reservation_id));

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

    fn probe(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ProbeReservations {
        let arrival_time = self.simulator.get_current_time_in_ms();
        let probe_request_answer = self.manager.probe_all_components(reservation_id);

        if probe_request_answer.is_empty() {
            if shadow_schedule_id.is_none() {
                self.log_state_probe(0, arrival_time);
            }
            return probe_request_answer;
        }

        if shadow_schedule_id.is_none() {
            self.log_state_probe(probe_request_answer.len() as i64, arrival_time);
        }

        return probe_request_answer;
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
        let arrival_time = self.simulator.get_current_time_in_ms();
        log::debug!(
            "Reserve: At VrmComponent {:?}, ReservationId {:?}, ShadowSchedule {:?}",
            self.id,
            self.reservation_store.get_name_for_key(reservation_id),
            shadow_schedule_id
        );

        // Can VrmComponents handle Request?
        if !self.manager.can_handel(reservation_id) {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);

            if shadow_schedule_id.is_none() {
                self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
            }
            return reservation_id;
        }

        // Perform Reserve
        if self.reservation_store.is_workflow(reservation_id) {
            // "Option Dance" with WorkflowScheduler
            if let Some(mut scheduler) = self.workflow_scheduler.take() {
                // Performs all reservation tracking like self.manager.not_committed_reservations
                scheduler.reserve(reservation_id, self);

                self.workflow_scheduler = Some(scheduler);
            } else {
                log::error!("WorkflowScheduler is missing or currently in use (recursive call?) for ADC {:?}", self.id);
                self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            }
        } else {
            // Atomic Job
            self.manager.reserve_task_at_first_grid_component(reservation_id, shadow_schedule_id.clone(), self.vrm_component_order);
        }

        // Check reservation
        if self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);

            if shadow_schedule_id.is_none() {
                self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
            }

            return reservation_id;
        }

        if shadow_schedule_id.is_none() {
            self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
        }
        return reservation_id;
    }
}
