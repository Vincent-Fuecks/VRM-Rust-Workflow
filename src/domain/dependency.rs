use crate::domain::reservation::{LinkReservation};

/// Represents an edge for data transfer (file).
#[derive(Debug, Clone)]
pub struct DataDependency {
    pub reservation: LinkReservation,
    pub source_node: String,
    pub target_node: String,
    pub port_name: String,
    pub size: i64,
}

/// Represents an edge for synchronous bandwidth.
#[derive(Debug, Clone)]
pub struct SyncDependency {
    pub reservation: LinkReservation,
    pub source_node: String,
    pub target_node: String,
    pub port_name: String,
    pub bandwidth: i64, 
}

/// An edge in the "overlay graph" connecting sync groups.
#[derive(Debug, Clone)]
pub struct OverlayDependency {
    // Underlying DataDependency ID
    pub id: String, 

    /// Key to Workflow.overlay_nodes
    pub source_overlay: String,
    
    /// Key to Workflow.overlay_nodes
    pub target_overlay: String,

    /// The DataDependency that this overlay edge represents
    pub data_dependency: String,
}