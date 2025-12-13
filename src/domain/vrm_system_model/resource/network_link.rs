use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;

#[derive(Debug, Clone)]
pub struct NetworkLink {
    pub id: ReservationKey,
    pub source: ReservationKey,
    pub target: ReservationKey,

    /// The schedule manages bandwidth for this link.
    pub schedule: SlottedSchedule,
}

impl NetworkLink {
    pub fn new(id: ReservationKey, source: ReservationKey, target: ReservationKey, schedule: SlottedSchedule) -> Self {
        Self { id: id, source: source, target: target, schedule: schedule }
    }
}
