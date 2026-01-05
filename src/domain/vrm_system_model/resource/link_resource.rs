use std::any::Any;
use std::collections::HashSet;

use crate::domain::vrm_system_model::reservation::link_reservation::LinkReservation;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
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

    fn can_handle(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        let link_source = reservation_store.get_start_point(reservation_id);
        let link_target = reservation_store.get_end_point(reservation_id);

        if link_source.is_none() || link_target.is_none() {
            return false;
        } else if self.source != link_source.unwrap() || self.target != link_target.unwrap() {
            return false;
        } else {
            return self.base.can_handle_capacity(reservation_store, reservation_id);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_id(&self) -> ResourceId {
        ResourceId::Link(self.base.get_id())
    }
}
