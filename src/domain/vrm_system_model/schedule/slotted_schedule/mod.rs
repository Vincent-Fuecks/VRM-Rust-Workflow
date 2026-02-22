use crate::domain::vrm_system_model::schedule::slotted_schedule::{
    slotted_schedule_context::SlottedScheduleContext,
    strategy::{link::link_strategy::LinkStrategy, node::node_strategy::NodeStrategy},
};

pub mod fragmentation;
pub mod schedule_base;
pub mod slot;
pub mod slotted_schedule_context;
pub mod strategy;

pub type SlottedScheduleNodes = SlottedScheduleContext<NodeStrategy>;
pub type SlottedScheduleLinks = SlottedScheduleContext<LinkStrategy>;
