use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::ResourceName;

use std::any::Any;

pub trait Resource: std::fmt::Debug + Send {
    /// Returns the capacity
    fn get_capacity(&self) -> i64;

    /// Down casting into NodeResource or LinkResource
    fn as_any(&self) -> &dyn Any;

    /// Return the Id of the Resource
    fn get_name(&self) -> ResourceName;
}
