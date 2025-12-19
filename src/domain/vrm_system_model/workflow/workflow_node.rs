use crate::domain::vrm_system_model::{
    reservation::node_reservation::NodeReservation,
    utils::id::{CoAllocationId, DataDependencyId, SyncDependencyId},
};

/// Represents a node in the workflow graph (a computation task).
#[derive(Debug, Clone)]
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
