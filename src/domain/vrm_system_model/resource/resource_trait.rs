use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::{LinkResourceId, NodeResourceId, RouterId};

use std::any::Any;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceId {
    Link(LinkResourceId),
    Node(NodeResourceId),
}

pub trait Resource: std::fmt::Debug + Send {
    /// Returns true if this specific resource can handle the reservation
    fn can_handle_adc_capacity_request(&self, res: Reservation) -> bool;

    /// Returns true if this specific resource can handle the reservation
    fn can_handle_aci_capacity_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool;

    /// Returns the capacity
    fn get_capacity(&self) -> i64;

    /// Down casting into NodeResource or LinkResource
    fn as_any(&self) -> &dyn Any;

    /// Return the Id of the Resource
    fn get_id(&self) -> ResourceId;
}
