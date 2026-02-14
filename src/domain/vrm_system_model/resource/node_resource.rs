use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::{resource_trait::Resource, resources::BaseResource};
use crate::domain::vrm_system_model::utils::id::{ResourceName, RouterId};

use std::any::Any;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct NodeResource {
    pub base: BaseResource,
}

impl NodeResource {
    pub fn new(name: ResourceName, capacity: i64, connected_routers: HashSet<RouterId>) -> Self {
        let base = BaseResource::new(name, capacity);
        Self { base }
    }
}

impl Resource for NodeResource {
    fn get_capacity(&self) -> i64 {
        self.base.capacity
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

    fn get_name(&self) -> ResourceName {
        self.base.get_name()
    }
}
