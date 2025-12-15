use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::schedule::topology::{NetworkTopology, Path};

use std::collections::HashMap;

/// Manages the schedule and reservations for the entire network grid.
pub struct NetworkSchedule {
    /// The underlying physical structure of the network, including routers, links,
    /// and pre-calculated virtual resources.
    pub topology: NetworkTopology,

    /// A registry mapping active reservations to their assigned physical paths.
    ///
    /// This map is essential for resolving which specific route a reservation utilizes,
    /// enabling conflict detection and bandwidth verification.
    reserved_paths: HashMap<ReservationKey, Path>,
}

impl NetworkSchedule {
    pub fn new(topology: NetworkTopology) -> Self {
        Self { topology, reserved_paths: HashMap::new() }
    }
}
