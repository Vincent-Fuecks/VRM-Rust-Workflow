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
