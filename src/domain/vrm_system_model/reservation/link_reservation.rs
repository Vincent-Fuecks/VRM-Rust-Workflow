use std::any::Any;

use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationBase, ReservationTrait, ReservationTyp};
use crate::domain::vrm_system_model::utils::id::RouterId;
/// This structure extends [`ReservationBase`] to include fields specific to
/// network connectivity.
///
/// Link reservations typically have two use cases:
/// 1. **Data Transfer:** Reserving bandwidth for file transfer between two sites.
///    In this case, the reservation may be **moldable**, meaning the duration
///    can be adjusted based on available bandwidth.
/// 2. **Co-allocated Communication:** Reserving a specific, fixed amount of
///    bandwidth for short-term coordination and communication between tasks
///    associated with co-allocated compute reservations. The specified bandwidth
///    **must** be provided for the entire duration.
#[derive(Debug, Clone)]
pub struct LinkReservation {
    /// The common base properties shared by all reservations.
    pub base: ReservationBase,

    // Link specific fields
    /// Unique identifier of the start router for the link.
    pub start_point: Option<RouterId>,
    /// Unique identifier of the end router for the link.
    pub end_point: Option<RouterId>,
}

impl ReservationTrait for LinkReservation {
    fn get_base(&self) -> &ReservationBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut ReservationBase {
        &mut self.base
    }

    fn box_clone(&self) -> Box<dyn ReservationTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_typ(&self) -> ReservationTyp {
        ReservationTyp::Link
    }
}
