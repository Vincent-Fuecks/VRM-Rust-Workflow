use crate::domain::vrm_system_model::adc::ADC;
use crate::domain::vrm_system_model::grid_resource_management_system::scheduler::workflow_scheduler::WorkflowSchedulerBase;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::workflow;
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
    pub fn reserve(&mut self, reservation_id: ReservationId, adc: &ADC, average_link_speed: i64) -> bool {
        if self.base.reservation_store.is_workflow(reservation_id) {
            log::error!("ReserveReservationTypError: ReservationTyp of reservation {:?} must be of typ Workflow", reservation_id);
            return false;
        }
        let ranked_node_reservations: Vec<WorkflowNode> =
            self.base.reservation_store.get_upward_rank(reservation_id, average_link_speed).expect("Should contain an Vector with WorkflowNodes.");
        let start = self.base.reservation_store.get_booking_interval_start(reservation_id);

        // for node in ranked_node_reservations {
        //     node.incoming_data
        // }
        todo!();
    }
}
