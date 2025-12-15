use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::utils::id::{NetworkLinkId, RouterId};

#[derive(Debug, Clone)]
pub struct NetworkLink {
    pub id: NetworkLinkId,
    pub source: RouterId,
    pub target: RouterId,
    pub bandwidth: i64,
    pub avg_bandwidth: i64,

    /// The schedule manages bandwidth for this link.
    pub schedule: SlottedSchedule,
}

impl NetworkLink {
    pub fn new(id: NetworkLinkId, source: RouterId, target: RouterId, bandwidth: i64, avg_bandwidth: i64, schedule: SlottedSchedule) -> Self {
        Self { id: id, source: source, target: target, bandwidth: bandwidth, avg_bandwidth: avg_bandwidth, schedule: schedule }
    }
}
