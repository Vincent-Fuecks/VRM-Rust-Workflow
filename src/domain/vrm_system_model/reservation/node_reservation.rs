use std::any::Any;

use serde::{Deserialize, Serialize};

use crate::domain::vrm_system_model::{
    reservation::reservation::{Reservation, ReservationBase, ReservationProceeding, ReservationState, ReservationTrait, ReservationTyp},
    utils::id::{ClientId, ComponentId, ReservationName},
};

/// This structure extends [`ReservationBase`] to include fields specific to
/// **computational node** (e.g., CPU cores).
///
/// The maximum task execution time (**duration**) has to be provided in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeReservation {
    /// The common base properties shared by all reservations.
    pub base: ReservationBase,

    // Node specific fields
    /// File system **path** pointing to the executable for this reservation/task.
    pub task_path: Option<String>,

    /// The file path where the **standard output** (stdout) during task execution will be piped.
    pub output_path: Option<String>,

    /// The file path where the **standard error** (stderr) during task execution will be piped.
    pub error_path: Option<String>,
}

impl NodeReservation {
    pub fn new(
        name: ReservationName,
        client_id: ClientId,
        handler_id: Option<ComponentId>,
        state: ReservationState,
        request_proceeding: ReservationProceeding,
        arrival_time: i64,
        booking_interval_start: i64,
        booking_interval_end: i64,
        task_duration: i64,
        reserved_capacity: i64,
        is_moldable: bool,
        frag_delta: f64,
        task_path: Option<String>,
        output_path: Option<String>,
        error_path: Option<String>,
    ) -> Self {
        // Calculate work: Capacity * Time
        let moldable_work = reserved_capacity * task_duration;

        let base = ReservationBase {
            name,
            client_id,
            handler_id,
            state,
            request_proceeding,
            arrival_time,
            booking_interval_start,
            booking_interval_end,
            assigned_start: 0, // Default to 0 until formally scheduled
            assigned_end: 0,   // Default to 0 until formally scheduled
            task_duration,
            reserved_capacity,
            is_moldable,
            moldable_work,
            frag_delta,
        };

        NodeReservation { base, task_path, output_path, error_path }
    }
}

impl ReservationTrait for NodeReservation {
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
        ReservationTyp::Node
    }
}
