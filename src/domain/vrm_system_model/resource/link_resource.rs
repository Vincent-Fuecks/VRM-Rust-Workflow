use std::any::Any;
use std::collections::HashSet;

use crate::domain::vrm_system_model::reservation::link_reservation::LinkReservation;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::resource::{
    resource_trait::{Resource, ResourceId},
    resources::BaseResource,
};
use crate::domain::vrm_system_model::schedule::slotted_schedule::SlottedSchedule;
use crate::domain::vrm_system_model::utils::id::{LinkResourceId, RouterId};

#[derive(Debug)]
pub struct LinkResource {
    base: BaseResource<LinkResourceId>,
    pub source: RouterId,
    pub target: RouterId,
    pub avg_bandwidth: i64,

    /// The schedule manages bandwidth for this link.
    pub schedule: SlottedSchedule,
}

impl LinkResource {
    pub fn new(
        id: LinkResourceId,
        connected_routers: HashSet<RouterId>,
        source: RouterId,
        target: RouterId,
        capacity: i64,
        avg_bandwidth: i64,
        schedule: SlottedSchedule,
    ) -> Self {
        let base = BaseResource::new(id, capacity, connected_routers);

        Self { base, source, target, avg_bandwidth, schedule }
    }
}

impl Resource for LinkResource {
    fn get_capacity(&self) -> i64 {
        self.base.capacity
    }

    fn get_connected_routers(&self) -> &HashSet<RouterId> {
        &self.base.connected_routers
    }

    fn can_handle(&self, reservation: &Box<dyn Reservation>) -> bool {
        // 1. Check Type (Java: instanceof LinkReservation)
        if let Some(link_res) = reservation.as_any().downcast_ref::<LinkReservation>() {
            // 2. Check Logic specific to Links
            if self.source != link_res.start_point || self.target != link_res.end_point {
                return false;
            }
        } else {
            // Not a LinkReservation
            return false;
        }

        // 3. Check Capacity (Java: super.canHandle)
        self.base.can_handle_capacity(reservation)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_id(&self) -> ResourceId {
        ResourceId::Link(self.base.get_id())
    }
}
