use crate::domain::reservation::{NodeReservation};

/// Represents a "sync group" of WorkflowNodes that must be scheduled together.
#[derive(Debug, Clone)]
pub struct OverlayNode {
    pub id: String,

    /// Keys to Workflow.nodes
    pub members: Vec<String>,
    
    // Rank for scheduling
    pub rank_upward: i64,
    pub rank_downward: i64,
    
    /// Keys to Workflow.overlay_dependencies
    pub incoming_overlay: Vec<String>,
    
    /// Keys to Workflow.overlay_dependencies
    pub outgoing_overlay: Vec<String>,
}

/// Represents a node in the workflow graph (a computation task).
#[derive(Debug, Clone)]
pub struct WorkflowNode {
    pub reservation: NodeReservation,

    /// Graph structure: Keys to the Workflow's HashMaps
    pub incoming_data: Vec<String>,
    pub outgoing_data: Vec<String>,
    pub incoming_sync: Vec<String>,
    pub outgoing_sync: Vec<String>,

    /// Key to the Workflow.overlay_nodes HashMap
    pub overlay_node: String,
}