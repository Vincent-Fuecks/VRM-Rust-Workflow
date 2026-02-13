use std::sync::Arc;

use crate::domain::{
    simulator::simulator::SystemSimulator,
    vrm_system_model::{
        reservation::reservation_store::ReservationStore, schedule::slotted_schedule::slotted_schedule::schedule_context::SlottedScheduleContext,
    },
};

pub mod fragmentation;
pub mod helper;
pub mod schedule;
pub mod schedule_context;
pub mod slot;

#[derive(Debug, Clone)]
pub struct SlottedSchedule {
    pub ctx: SlottedScheduleContext,
    pub capacity: i64,
    pub reservation_store: ReservationStore,
    simulator: Arc<dyn SystemSimulator>,
}

impl SlottedSchedule {
    pub fn new(ctx: SlottedScheduleContext, capacity: i64, reservation_store: ReservationStore, simulator: Arc<dyn SystemSimulator>) -> Self {
        Self { ctx, capacity, reservation_store, simulator }
    }
}
