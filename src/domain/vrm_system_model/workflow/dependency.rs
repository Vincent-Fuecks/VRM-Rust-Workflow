use serde::{Deserialize, Serialize};

use crate::domain::vrm_system_model::{
    reservation::link_reservation::LinkReservation,
    utils::id::{CoAllocationDependencyId, CoAllocationId, DataDependencyId, WorkflowNodeId},
};

/// Represents an edge for data transfer (file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDependency {
    /// Contains common properties shared by all reservations and includes specific fields for network connectivity.
    pub reservation: LinkReservation,

    /// Key to Workflow.nodes, which is the sender.
    pub source_node: Option<WorkflowNodeId>,

    /// Key to Workflow.nodes, which is the receiver.
    pub target_node: Option<WorkflowNodeId>,

    /// TODO
    pub port_name: String,

    /// TODO Size of the file for transport?
    pub size: i64,
}

/// Represents an edge for synchronous bandwidth (e.g. Co-allocated Communication).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDependency {
    /// Contains common properties shared by all reservations and includes specific fields for network connectivity.
    pub reservation: LinkReservation,

    /// Key to Workflow.nodes, which is the sender.
    pub source_node: Option<WorkflowNodeId>,

    /// Key to Workflow.nodes, which is the receiver.
    pub target_node: Option<WorkflowNodeId>,

    /// TODO
    pub port_name: String,

    /// Bandwidth in MB's
    pub bandwidth: i64,
}

/// An edge in the "CoAllocations graph" connecting sync groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoAllocationDependency {
    // Underlying DataDependency ID
    pub id: CoAllocationDependencyId,

    /// Key to Workflow.co_allocations, which is the sender.
    pub source_group: CoAllocationId,

    /// Key to Workflow.co_allocations, which is the receiver.
    pub target_group: CoAllocationId,

    /// Key to the DataDependency that this CoAllocation edge represents.
    pub data_dependency: DataDependencyId,
}
