use std::{collections::HashMap, sync::Arc};

use crate::domain::{
    simulator::simulator::SystemSimulator,
    vrm_system_model::{
        reservation::reservation_store::{ReservationId, ReservationStore},
        resource::resource_store::ResourceStore,
        schedule::slotted_schedule::{
            network_slotted_schedule::topology::{NetworkTopology, Path},
            slotted_schedule::schedule_context::SlottedScheduleContext,
        },
    },
};

pub mod helper;
pub mod schedule;
pub mod topology;

/// Creates the schedule for Networks like NullBroker, SLURM etc.
/// Shares with the SlottedSchedule the SlottedScheduleContext and multiple other function
/// of the implemented Schedule trait.
#[derive(Debug, Clone)]
pub struct NetworkSlottedSchedule {
    pub ctx: SlottedScheduleContext,
    pub topology: NetworkTopology,
    pub reserved_paths: HashMap<ReservationId, HashMap<i64, Path>>,
    pub reservation_store: ReservationStore,
    pub resource_store: ResourceStore,
    simulator: Arc<dyn SystemSimulator>,
}

impl NetworkSlottedSchedule {
    pub fn new(
        ctx: SlottedScheduleContext,
        topology: NetworkTopology,
        reservation_store: ReservationStore,
        resource_store: ResourceStore,
        simulator: Arc<dyn SystemSimulator>,
    ) -> Self {
        Self { ctx, topology, reserved_paths: HashMap::new(), reservation_store, resource_store, simulator }
    }
}
