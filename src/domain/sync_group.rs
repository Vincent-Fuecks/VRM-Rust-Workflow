use crate::domain::dependency::{DataDependency, SyncDependency, SyncGroupDependency};
use crate::domain::workflow_node::WorkflowNode;

/// Represents a "sync group" of WorkflowNodes that must be scheduled together.
/// Contains a goup fo Workflow nodes, which are connected by sync dependencies.
///
#[derive(Debug, Clone)]
pub struct SyncGroup {
    pub id: String,

    /// WorkflowNode, which is representing all WorkflowNodes in this sync group
    pub representative: Option<WorkflowNode>,

    /// Keys to Workflow.nodes, which are part of this SyncGroup.
    pub members: Vec<String>,

    /// SyncDependencies connecting WorkflowNodes of this SyncGroup.
    pub sync_dependencies: Vec<SyncDependency>,

    // TODO Should be maybe done per WorkflowNode Ids? Maybe better?
    pub outgoing_sync_group_dependencies: Vec<SyncGroupDependency>,
    pub outgoing_data_dependencies: Vec<DataDependency>,

    pub incoming_sync_group_dependencies: Vec<SyncGroupDependency>,
    pub incoming_data_dependencies: Vec<DataDependency>,

    // Rank for scheduling
    // TODO What is the overlayNode the representative?
    /**
     * Upward Rank for this OverlayNode. That is the length of the longest path through the Workflow,
     * starting with the Overlay node and ending at an exit node. The DataDependencies between nodes are considered,
     * using the average communication speed of the used network.
     * rank_u(n_i) = w_i + max(n_j elem succ(n_i))(c_ij + rank_u(n_j)) where
     * n elem node
     * w_i computation time of node i
     * c_ij average data transfer time from n_i to n_j  
     */
    pub rank_upward: i64,

    // TODO What is the overlayNode the representative?
    /**
     * Downward Rank for this OverlayNode. That is the length of the longest path through the Workflow,
     * starting at an entry node and ending at the Overlay node. The DataDependencies between nodes are considered,
     * using the average communication speed of the used network.
     * rank_d(n_i) = max(n_j elem pred(n_i))(rank_d(n_j) + w_j + c_ji) where
     * n elem node
     * w_j computation time of node j
     * c_ji average data transfer time from n_j to n_i  
     */
    pub rank_downward: i64,

    /// TODO Number of nodes on the critical path from an entry node to this node.
    pub number_of_nodes_critical_path_downwards: i64,

    /// TODO Number of nodes on the critical path from this node to the exit node, including this node.
    pub number_of_nodes_critical_path_upwards: i64,

    // Temporary calculation values (internal state)
    /// Temporary value for topological order calculation (isInQueu).
    pub is_in_queue: bool,

    /// Temporary value for topological order calculation (unprocessedPredecessors).
    pub unprocessed_predecessors: i64,

    /// Spare time distributed temporary to this SyncGroup during calculation of booking interval.
    pub spare_time: i64,

    // FRAG-WINDOW Scheduling forces/properties
    /// Force for deadline positioning during FRAG-WINDOW scheduling algorithm.
    pub max_succ_force: f64,

    /// Force for deadline positioning during FRAG-WINDOW scheduling algorithm.
    pub max_pred_force: f64,

    // Search flags
    /// Mark flag which can be used by search algorithms.
    pub is_discovered: bool,

    /// Mark flag which can be used by search algorithms.
    pub is_processed: bool,

    // TODO
    pub is_moveable: bool,
    pub is_moveable_interval_start: bool,
    pub is_moveable_interval_end: bool,
    pub start_position: f64,
    pub end_position: f64,
}
