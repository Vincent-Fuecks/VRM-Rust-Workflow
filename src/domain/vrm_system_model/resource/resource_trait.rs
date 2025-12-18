use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::utils::id::{LinkResourceId, NodeResourceId, RouterId};

use std::any::Any;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceId {
    Link(LinkResourceId),
    Node(NodeResourceId),
}

pub trait Resource: std::fmt::Debug {
    /// Returns true if this specific resource can handle the reservation
    fn can_handle(&self, reservation: &Box<dyn Reservation>) -> bool;

    /// Returns the capacity
    fn get_capacity(&self) -> i64;

    /// Returns connected routers
    fn get_connected_routers(&self) -> &HashSet<RouterId>;

    /// Down casting into NodeResource or LinkResource
    fn as_any(&self) -> &dyn Any;

    /// Return the Id of the Resource
    fn get_id(&self) -> ResourceId;
}
