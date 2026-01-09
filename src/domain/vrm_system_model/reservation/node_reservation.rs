use std::any::Any;

use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationBase, ReservationTrait, ReservationTyp};

/// This structure extends [`ReservationBase`] to include fields specific to
/// **computational node** (e.g., CPU cores).
///
/// The maximum task execution time (**duration**) has to be provided in advance.
#[derive(Debug, Clone)]
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
