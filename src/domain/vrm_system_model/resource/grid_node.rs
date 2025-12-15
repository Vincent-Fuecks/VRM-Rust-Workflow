use crate::domain::vrm_system_model::utils::id::{GridNodeId, RouterId};

#[derive(Debug, Clone)]
pub struct GridNode {
    pub id: GridNodeId,
    pub cpus: i64,
    pub connected_to_router: Vec<RouterId>,
}
