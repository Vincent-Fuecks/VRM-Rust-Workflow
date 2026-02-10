use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::NetworkTopology;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule::schedule_context::SlottedScheduleContext;
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::utils::id::SlottedScheduleId;

use crate::error::ConversionError;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum SchedulerType {
    // Node Scheduler
    FreeListSchedule,
    SlottedSchedule,

    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,

    // Network Scheduler
    SlottedScheduleNetwork { topology: NetworkTopology },
}
#[derive(Debug, Clone)]
pub struct ScheduleContext {
    pub id: SlottedScheduleId,
    pub number_of_slots: i64,
    pub slot_width: i64,
    pub capacity: i64,
    pub simulator: Arc<dyn SystemSimulator>,
    pub reservation_store: ReservationStore,
}

impl FromStr for SchedulerType {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FreeListSchedule" => Ok(SchedulerType::FreeListSchedule),
            "SlottedSchedule" => Ok(SchedulerType::SlottedSchedule),
            "SlottedScheduleResubmitFrag" => Ok(SchedulerType::SlottedScheduleResubmitFrag),
            "SlottedSchedule12" => Ok(SchedulerType::SlottedSchedule12),
            "SlottedSchedule12000" => Ok(SchedulerType::SlottedSchedule12000),
            "UnlimitedSchedule" => Ok(SchedulerType::UnlimitedSchedule),
            _ => Err(ConversionError::UnknownSchedulerType(s.to_string())),
        }
    }
}

impl SchedulerType {
    // Factory method to create a concrete Schedule implementation
    pub fn get_instance(&self, ctx: ScheduleContext) -> Box<dyn Schedule> {
        match self {
            Self::FreeListSchedule => {
                todo!()
            }
            Self::SlottedSchedule => {
                let slotted_schedule_ctx = SlottedScheduleContext::new(
                    ctx.id,
                    ctx.simulator.get_current_time_in_s(),
                    ctx.number_of_slots,
                    ctx.slot_width,
                    ctx.capacity,
                    true,
                    ctx.reservation_store.clone(),
                );

                Box::new(SlottedSchedule::new(slotted_schedule_ctx, ctx.capacity, ctx.reservation_store, ctx.simulator))
            }
            Self::SlottedScheduleNetwork { topology } => {
                let slotted_schedule_ctx = SlottedScheduleContext::new(
                    ctx.id,
                    ctx.simulator.get_current_time_in_s(),
                    ctx.number_of_slots,
                    ctx.slot_width,
                    ctx.capacity,
                    true,
                    ctx.reservation_store.clone(),
                );

                Box::new(SlottedSchedule::new(slotted_schedule_ctx, ctx.capacity, ctx.reservation_store, ctx.simulator))
            }
            Self::SlottedSchedule12 => {
                let number_of_real_slots = (ctx.number_of_slots * (ctx.slot_width + 11)) / 12;
                let slotted_schedule_ctx = SlottedScheduleContext::new(
                    ctx.id,
                    ctx.simulator.get_current_time_in_s(),
                    number_of_real_slots,
                    12,
                    ctx.capacity,
                    true,
                    ctx.reservation_store.clone(),
                );

                Box::new(SlottedSchedule::new(slotted_schedule_ctx, ctx.capacity, ctx.reservation_store, ctx.simulator))
            }
            Self::SlottedSchedule12000 => {
                let number_of_real_slots = (ctx.number_of_slots * (ctx.slot_width + 11999)) / 12000;
                let slotted_schedule_ctx = SlottedScheduleContext::new(
                    ctx.id,
                    ctx.simulator.get_current_time_in_s(),
                    number_of_real_slots,
                    1200,
                    ctx.capacity,
                    true,
                    ctx.reservation_store.clone(),
                );

                Box::new(SlottedSchedule::new(slotted_schedule_ctx, ctx.capacity, ctx.reservation_store, ctx.simulator))
            }
            Self::SlottedScheduleResubmitFrag => {
                let slotted_schedule_ctx = SlottedScheduleContext::new(
                    ctx.id,
                    ctx.simulator.get_current_time_in_s(),
                    ctx.number_of_slots,
                    ctx.slot_width,
                    ctx.capacity,
                    false,
                    ctx.reservation_store.clone(),
                );

                Box::new(SlottedSchedule::new(slotted_schedule_ctx, ctx.capacity, ctx.reservation_store, ctx.simulator))
            }
            Self::UnlimitedSchedule => {
                todo!()
            }
        }
    }

    pub fn get_network_scheduler_variant(&self, topology: NetworkTopology) -> SchedulerType {
        match self {
            Self::SlottedSchedule => SchedulerType::SlottedScheduleNetwork { topology },
            _ => {
                log::error!("The specified Scheduler {:?} is not implemented as NetworkScheduler. Default to SlottedSchedule", self);
                SchedulerType::SlottedScheduleNetwork { topology }
            }
        }
    }
}
