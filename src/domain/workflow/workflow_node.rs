use crate::domain::workflow::reservation::NodeReservation;

/// Represents a node in the workflow graph (a computation task).
#[derive(Debug, Clone)]
pub struct WorkflowNode {
    pub reservation: NodeReservation,

    /// Graph structure: Keys to the Workflow's HashMaps
    pub incoming_data: Vec<String>,
    pub outgoing_data: Vec<String>,
    pub incoming_sync: Vec<String>,
    pub outgoing_sync: Vec<String>,

    /// Key of the Workflow.co_allocations HashMap.
    /// HashMap contains all other nodes in the same sync group, including this node.
    pub co_allocation_key: String,
}
