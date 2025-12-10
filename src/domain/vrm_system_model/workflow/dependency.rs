use crate::domain::vrm_system_model::reservation::{
    link_reservation::LinkReservation, reservation::ReservationKey,
};

// TODO Should be used as Keys in workflows.rs
// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// pub enum DependencyType {
//     Data,
//     Sync,
// }

// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
// struct DependencyKey {
//     task_id: String,
//     port_name: String,
//     typ: DependencyType, // The string field
// }

/// Represents an edge for data transfer (file).
#[derive(Debug, Clone)]
pub struct DataDependency {
    /// Contains common properties shared by all reservations and includes specific fields for network connectivity.
    pub reservation: LinkReservation,

    /// Key to Workflow.nodes, which is the sender.
    pub source_node: ReservationKey,

    /// Key to Workflow.nodes, which is the receiver.
    pub target_node: ReservationKey,

    /// TODO
    pub port_name: String,

    /// TODO Size of the file for transport?
    pub size: i64,
}

/// Represents an edge for synchronous bandwidth (e.g. Co-allocated Communication).
#[derive(Debug, Clone)]
pub struct SyncDependency {
    /// Contains common properties shared by all reservations and includes specific fields for network connectivity.
    pub reservation: LinkReservation,

    /// Key to Workflow.nodes, which is the sender.
    pub source_node: ReservationKey,

    /// Key to Workflow.nodes, which is the receiver.
    pub target_node: ReservationKey,

    /// TODO
    pub port_name: String,

    /// Bandwidth in Mbps
    pub bandwidth: i64,
}

/// An edge in the "CoAllocations graph" connecting sync groups.
#[derive(Debug, Clone)]
pub struct CoAllocationDependency {
    // Underlying DataDependency ID
    pub id: ReservationKey,

    /// Key to Workflow.co_allocations, which is the sender.
    pub source_group: ReservationKey,

    /// Key to Workflow.co_allocations, which is the receiver.
    pub target_group: ReservationKey,

    /// Key to the DataDependency that this CoAllocation edge represents.
    pub data_dependency: ReservationKey,
}
