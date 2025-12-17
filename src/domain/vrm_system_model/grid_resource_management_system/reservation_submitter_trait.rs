use crate::domain::vrm_system_model::grid_resource_management_system::reservation_processor_trait::ReservationProcessor;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;

/// The communication interface for any component that submits tasks to a
/// [`GridResourceManagementSystem`].
///
/// In a **Distributed Resource Management (VRM)** environment, task execution and
/// state transitions (such as moving from `Reserved` to `Committed` or `Finished`)
/// often happen asynchronously. This trait allows the system to push these
/// updates back to the submitting client.
pub trait ReservationSubmitter {
    /// Callback method invoked by the [`ExtendedReservationProcessor`] to inform
    /// the submitter that a reservation's state has changed.
    ///
    /// This is triggered during critical lifecycle events, such as:
    /// * **Successful Completion**: The task reached `ReservationState::Finished`.
    /// * **Failure/Rejection**: The system encountered an error or a timeout occurred.
    /// * **Preemption**: The resource was reclaimed by a higher-priority task.
    ///
    /// # Arguments
    /// * `sender` - A reference to the resource management system reporting the change.
    /// * `changed_reservation` - A `Reservation` object containing the updated state.
    ///   While this may be a different instance than the one initially submitted,
    ///   the `task_name` remains the unique invariant for identification.
    /// TODO Rework if implemented --> Is necessary
    fn state_changed(&self, sender: &dyn ReservationProcessor, changed_reservation: Box<dyn Reservation>);
}
