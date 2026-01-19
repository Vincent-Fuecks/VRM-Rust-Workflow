use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::{
    resource_trait::{Resource, ResourceId},
    resources::BaseResource,
};
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

    fn can_handle_adc_capacity_request(&self, res: Reservation) -> bool {
        self.base.can_handle_adc_capacity_request(res)
    }

    fn can_handle_aci_capacity_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        self.base.can_handle_aci_capacity_request(reservation_store, reservation_id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_id(&self) -> ResourceId {
        ResourceId::Node(self.base.get_id())
    }
}
