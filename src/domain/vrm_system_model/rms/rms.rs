use crate::domain::vrm_system_model::resource::{grid_node::GridNode, network_link::NetworkLink};
use crate::domain::vrm_system_model::scheduler_trait::Schedule;

use std::any::Any;

pub trait Rms: std::fmt::Debug + Any + Send + Sync {
    fn get_base(&self) -> &RmsBase;

    fn get_base_mut(&mut self) -> &mut RmsBase;

    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug)]
pub struct RmsBase {
    schedule: Box<dyn Schedule>,
    grid_nodes: Vec<GridNode>,
    network_links: Vec<NetworkLink>,
    slot_width: i64,
    num_of_slots: i64,
}
