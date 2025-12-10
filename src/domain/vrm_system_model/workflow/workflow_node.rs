use crate::domain::vrm_system_model::reservation::{
    node_reservation::NodeReservation, reservation::ReservationKey,
};

/// Represents a node in the workflow graph (a computation task).
#[derive(Debug, Clone)]
pub struct WorkflowNode {
    pub reservation: NodeReservation,

    /// Graph structure: Keys to the Workflow's HashMaps
    pub incoming_data: Vec<ReservationKey>,
    pub outgoing_data: Vec<ReservationKey>,
    pub incoming_sync: Vec<ReservationKey>,
    pub outgoing_sync: Vec<ReservationKey>,

    /// Key of the Workflow.co_allocations HashMap.
    /// HashMap contains all other nodes in the same sync group, including this node.
    pub co_allocation_key: ReservationKey,
}
