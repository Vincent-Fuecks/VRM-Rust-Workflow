use crate::domain::reservation::{LinkReservation};

/// Represents an edge for data transfer (file).
#[derive(Debug, Clone)]
pub struct DataDependency {
    /// Contains common properties shared by all reservations and includes specific fields for network connectivity.
    pub reservation: LinkReservation,
    
    /// Key to Workflow.nodes, which is the sender. 
    pub source_node: String,

    /// Key to Workflow.nodes, which is the receiver. 
    pub target_node: String,
    
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
    pub source_node: String,

    /// Key to Workflow.nodes, which is the receiver. 
    pub target_node: String,

    /// TODO
    pub port_name: String,

    /// Bandwidth in Mbps
    pub bandwidth: i64, 
}

/// An edge in the "overlay graph" connecting sync groups.
/// TODO Adjust comments after rework of OverlayNode -> SyncGroup 
#[derive(Debug, Clone)]
pub struct SyncGroupDependency {
    // Underlying DataDependency ID
    pub id: String, 

    /// Key to Workflow.overlay_nodes, which is the sender. 
    pub source_overlay_node: String,
    
    /// Key to Workflow.overlay_nodes, which is the receiver. 
    pub target_overlay_node: String,

    /// Key to the DataDependency that this overlay edge represents.
    pub data_dependency: String,
}