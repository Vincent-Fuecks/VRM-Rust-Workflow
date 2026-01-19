use std::collections::{HashMap, HashSet};

use crate::domain::vrm_system_model::grid_resource_management_system::adc::ADC;
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::grid_resource_management_system::scheduler::workflow_scheduler::WorkflowSchedulerBase;
use crate::domain::vrm_system_model::grid_resource_management_system::scheduler_comparator::eft_reservation_compare::EFTReservationCompare;
use crate::domain::vrm_system_model::reservation::node_reservation::NodeReservation;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState, ReservationTrait};
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::{ComponentId, ComponentTag};
use crate::domain::vrm_system_model::workflow;
use crate::domain::vrm_system_model::workflow::workflow::Workflow;
use crate::domain::vrm_system_model::workflow::workflow_node::WorkflowNode;
/**
 * Scheduler using the HEFTSync workflow scheduling algorithm.
 *
 * The basic idea of the scheduling algorithm is a list scheduler
 * using the length of the critical path assuming average resources
 * as list sorting criteria. For each job in the resource providing
 * the earliest finishing time (EFT) will be selected. This algorithm
 * was extended to cope with co-allocations (synchronous dependencies
 * in the VRM).
 *
 * @see ADC
 * @see Workflow
 */

pub struct HEFTSyncWorkflowScheduler {
    pub base: WorkflowSchedulerBase,
}

impl HEFTSyncWorkflowScheduler {
    pub fn reserve(&mut self, reservation_id: ReservationId, adc: &mut ADC, average_link_speed: i64) -> bool {
        // 1. Get exclusive access via the store
        if let Some(workflow_handle) = self.base.reservation_store.get(reservation_id) {
            let mut reservation = workflow_handle.write().unwrap();

            let mut grid_component_res_database: HashMap<ComponentId, HashSet<ReservationId>> = HashMap::new();

            if let Reservation::Workflow(ref mut workflow) = *reservation {
                let ranked_node_reservations = workflow.calculate_upward_rank(average_link_speed, &self.base.reservation_store);

                let workflow_booking_interval_end = workflow.get_booking_interval_end();

                for mut workflow_node in ranked_node_reservations {
                    let mut start = workflow.get_booking_interval_start();

                    let co_allocation_key = &workflow_node.co_allocation_key.clone().unwrap();
                    let co_allocation_node = workflow.co_allocations.get(co_allocation_key).unwrap();

                    for data_dependency in co_allocation_node.incoming_data_dependencies.clone() {
                        let data_dep_source_res_id = data_dependency.source_node.unwrap();

                        let data_dep_source_assigned_end =
                            self.base.reservation_store.get_assigned_end(workflow.nodes.get(&data_dep_source_res_id).unwrap().reservation_id);

                        let mut file_transfer_time = data_dependency.size / average_link_speed;

                        // If there is something to transfer it should be at least be one
                        if data_dependency.size > 0 && file_transfer_time == 0 {
                            log::debug!(
                                "MissMatchDataDependencySizeTransferTime: The Data dependency {} has a size of {}, however the file transfer time is 0. Process dependency with transfer_time of 1.",
                                self.base.reservation_store.get_name_for_key(data_dependency.reservation_id).unwrap(),
                                data_dependency.size
                            );
                            file_transfer_time = 1;
                        }

                        let start_after_this_dep = data_dep_source_assigned_end + file_transfer_time;

                        if start_after_this_dep > start {
                            start = start_after_this_dep;
                        }
                    }
                    // Access duration from Store
                    let task_duration = self.base.reservation_store.get_task_duration(workflow_node.reservation_id);
                    let node_name = self.base.reservation_store.get_name_for_key(workflow_node.reservation_id).unwrap();

                    // Do not process workflow, where the deadline will be missed
                    if start + task_duration > workflow_booking_interval_end {
                        log::debug!(
                            "No valid schedule found reservation {} of workflow {}, due to missed deadline.",
                            node_name,
                            workflow.base.get_name()
                        );
                        self.cancel_all_reservations(adc, &grid_component_res_database);
                        self.base.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                        return false;
                    }

                    self.base.reservation_store.set_booking_interval_start(workflow_node.reservation_id, start);
                    // Possible improvement: Could be shortened by node rank
                    self.base.reservation_store.set_booking_interval_end(workflow_node.reservation_id, workflow_booking_interval_end);

                    // Schedule all compute task (and all synced compute tasks and sync dependencies)
                    if (!self.schedule_co_allocation_node_reservations(workflow, &mut workflow_node, &mut grid_component_res_database, adc)) {
                        self.cancel_all_reservations(adc, &grid_component_res_database);
                        workflow.set_state(ReservationState::Rejected);
                        return false;
                    }

                    // Try to get network connection form all predecessors
                    if (!self.schedule_data_dependencies(&workflow, &mut workflow_node, adc)) {
                        self.cancel_all_reservations(adc, &grid_component_res_database);
                        workflow.set_state(ReservationState::Rejected);
                        return false;
                    }
                }

                // Inform ADC about the done Reservations
                adc.register_workflow_subtasks(&workflow, &grid_component_res_database);
                workflow.set_state(ReservationState::ReserveAnswer);
                return true;
            }
        }
        return false;
    }

    /**
     * Schedule and try to reserve all data dependencies (e.g. file transfers) to
     * all {@link NodeReservation}s co-allocated with the given reservation. All
     * predecessor have to be scheduled/reserved.
     *
     * @param workflow The workflow containing the relations between all reservations
     * @param mainTargetRes A representative for a set of {@link NodeReservation}s which are connected by sync dependencies
     * @param aisPerReservation A container for all successful reservations for this workflow
     */

    fn schedule_data_dependencies(&mut self, workflow: &Workflow, workflow_node: &mut WorkflowNode, adc: &mut ADC) -> bool {
        todo!()
    }

    fn schedule_co_allocation_node_reservations(
        &mut self,
        workflow: &mut Workflow,
        node_to_schedule: &mut WorkflowNode,
        grid_component_res_database: &mut HashMap<ComponentId, HashSet<ReservationId>>,
        adc: &mut ADC,
    ) -> bool {
        let co_allocation_to_schedule = node_to_schedule.co_allocation_key.clone().unwrap();
        let mut co_allocation_nodes_to_schedule = workflow.co_allocations.get(&co_allocation_to_schedule).unwrap().members.clone();

        let reservation_id_to_schedule = node_to_schedule.reservation_id;

        let first_task_candidate = self.schedule_node_reservation_eft(workflow, reservation_id_to_schedule, grid_component_res_database, adc);

        // Failure
        if first_task_candidate.is_none()
            || self.base.reservation_store.is_reservation_state_at_least(first_task_candidate.unwrap(), ReservationState::ReserveAnswer)
        {
            return false;
        }
        let first_task_candidate = first_task_candidate.unwrap();

        // Updates time boundaries
        workflow.update_reservation(self.base.reservation_store.clone(), first_task_candidate);

        // Get Co-Allocation constrains
        let duration = self.base.reservation_store.get_task_duration(first_task_candidate);
        let start = self.base.reservation_store.get_assigned_start(first_task_candidate);
        let end = self.base.reservation_store.get_assigned_end(first_task_candidate);

        // All nodes which are connected by Sync dependencies
        // Update all group members of Co-Allocation Node
        for co_allocation_node_id in co_allocation_nodes_to_schedule {
            let member_id = workflow.nodes.get(&co_allocation_node_id).unwrap().reservation_id;
            self.base.reservation_store.set_booking_interval_start(member_id, start);
            self.base.reservation_store.set_booking_interval_end(member_id, end);
            self.base.reservation_store.adjust_capacity(member_id, duration);

            // Try to reserve this task
            // TODO
        }
        todo!()
    }

    /**
     * Schedule and reserve a network link for the given dependency.
     *
     * @param workflow The workflow containing the relations between all reservations
     * @param dependency The dependency to schedule
     * @param start     earliest start time in s (VRM time)
     * @param end       latest end time in s (VRM time)
     * @param sourceAI  AI where the network transfer starts
     * @param targetAI  AI where the network transfer ends
     * @param isFiletransfer <code>true</code>, if it is a file transfer which is moldable. <code>false</code>, if it is a synchronous channel with fixed timing and bandwidth
     * @param aisPerReservation A container for all successful reservations for this workflow
     */
    fn schedule_dependency() -> bool {
        todo!()
    }

    /**
     * Schedule and try to reserve the given reservation such that it finish
     * as early as possible (EFT).
     *
     * @param workflow The workflow containing the relations between all reservations
     * @param nodeToSchedule the reservation to schedule
     * @param aisPerReservation A container for all successful reservations for this workflow
     */
    fn schedule_node_reservation_eft(
        &self,
        workflow: &mut Workflow,
        reservation_id: ReservationId,
        grid_component_res_database: &mut HashMap<ComponentId, HashSet<ReservationId>>,
        adc: &mut ADC,
    ) -> Option<ReservationId> {
        // Request all GirdComponents for reservation candidates and sort them according to EFT (earliest finishing time)

        let comparator = EFTReservationCompare::new(self.base.reservation_store.clone());

        let reservation_order = move |id0: ReservationId, id1: ReservationId| comparator.compare(id0, id1);

        let candidate_id = adc.submit_task_at_best_aci(reservation_id, None, grid_component_res_database, reservation_order);

        if !candidate_id.is_none()
            && self.base.reservation_store.is_reservation_state_at_least(candidate_id.unwrap(), ReservationState::ReserveAnswer)
        {
            workflow.update_reservation(self.base.reservation_store.clone(), candidate_id.unwrap());
            return candidate_id;
        }
        return None;
    }

    /**
     * Cancels all reservations of a workflow already done.
     *
     * @param aisPerReservation a container with all reservations to cancel and the AIs where they are booked.
     */
    pub fn cancel_all_reservations(&mut self, adc: &mut ADC, grid_component_res_database: &HashMap<ComponentId, HashSet<ReservationId>>) {
        for (component_id, reservation_id_set) in grid_component_res_database {
            adc.delete_tasks_at_component(component_id.clone(), reservation_id_set.clone(), None)
        }
    }
}
