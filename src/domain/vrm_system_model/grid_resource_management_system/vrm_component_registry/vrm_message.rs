use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{ComponentId, RouterId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

use std::sync::mpsc;

/// Messages representing every method in the VrmComponent trait.
/// These allow us to serialize method calls across threads.
pub enum VrmMessage {
    GetId(mpsc::Sender<ComponentId>),
    GetTotalCapacity(mpsc::Sender<i64>),
    GetTotalLinkCapacity(mpsc::Sender<i64>),
    GetTotalNodeCapacity(mpsc::Sender<i64>),
    GetLinkResourceCount(mpsc::Sender<usize>),
    GetRouterList(mpsc::Sender<Vec<RouterId>>),

    CanHandel { reservation: Reservation, reply_to: mpsc::Sender<bool> },

    Probe { reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<Reservations> },

    // TODO not possible in this form
    Reserve { reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<ReservationId> },

    Commit { reservation_id: ReservationId, reply_to: mpsc::Sender<bool> },

    DeleteTask { reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<ReservationId> },

    GetSatisfaction { start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<f64> },

    GetSystemSatisfaction { shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<f64> },

    CreateShadowSchedule { id: ShadowScheduleId, reply_to: mpsc::Sender<bool> },

    DeleteShadowSchedule { id: ShadowScheduleId, reply_to: mpsc::Sender<bool> },

    CommitShadowSchedule { id: ShadowScheduleId, reply_to: mpsc::Sender<bool> },

    GetLoadMetricUpToDate { start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<LoadMetric> },

    GetLoadMetric { start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<LoadMetric> },

    GetSimulationLoadMetric { shadow_schedule_id: Option<ShadowScheduleId>, reply_to: mpsc::Sender<LoadMetric> },

    Shutdown,
}
