use serde::{Deserialize, Serialize};

use crate::domain::vrm_system_model::{
    reservation::node_reservation::NodeReservation,
    utils::id::{CoAllocationId, DataDependencyId, SyncDependencyId},
    workflow::workflow::Workflow,
};

/// Represents a node in the workflow graph (a computation task).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub reservation: NodeReservation,

    /// Graph structure: Keys to the Workflow's HashMaps
    pub incoming_data: Vec<DataDependencyId>,
    pub outgoing_data: Vec<DataDependencyId>,
    pub incoming_sync: Vec<SyncDependencyId>,
    pub outgoing_sync: Vec<SyncDependencyId>,

    /// Key of the Workflow.co_allocations HashMap.
    /// HashMap contains all other nodes in the same sync group, including this node.
    pub co_allocation_key: Option<CoAllocationId>,
}

impl WorkflowNode {
    pub fn update_reservation(&mut self, workflow: &mut Workflow) {
        if workflow.base.assigned_start == i64::MIN
            || (self.reservation.base.assigned_start < workflow.base.assigned_start && self.reservation.base.assigned_start != i64::MIN)
        {
            workflow.base.set_assigned_start(self.reservation.base.assigned_start);
        }

        if workflow.base.assigned_end == i64::MIN || self.reservation.base.assigned_end > workflow.base.assigned_end {
            workflow.base.set_assigned_end(self.reservation.base.assigned_end);
        }
    }
}
