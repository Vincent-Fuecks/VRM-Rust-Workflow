use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::resource::{resource_trait::Resource, resources::BaseResource};
use crate::domain::vrm_system_model::utils::id::{NodeResourceId, RouterId};

use std::any::Any;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct NodeResource {
    pub base: BaseResource<NodeResourceId>,
}

impl NodeResource {
    pub fn new(id: NodeResourceId, capacity: i64, connected_routers: HashSet<RouterId>) -> Self {
        let base = BaseResource::new(id, capacity, connected_routers);
        Self { base }
    }
}

impl Resource for NodeResource {
    fn get_capacity(&self) -> i64 {
        self.base.capacity
    }

    fn get_connected_routers(&self) -> &HashSet<RouterId> {
        &self.base.connected_routers
    }

    fn can_handle(&self, reservation: &Box<dyn Reservation>) -> bool {
        self.base.can_handle_capacity(reservation)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
