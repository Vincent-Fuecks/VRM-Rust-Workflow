use crate::domain::vrm_system_model::grid_resource_management_system::reservation_submitter_trait::ReservationSubmitter;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;

/// A specialized interface for a fully-featured **Distributed Resource Management System**.
///
/// This trait is designed for a simple **Grid Resource Management Systems (GRMS)** and adapters
/// connecting to local resource management systems.
///
/// ### The Three Levels of Commitment
///
/// In a distributed Grid environment, managing resource state requires a clear lifecycle:
/// 1. **Probe**: A non-binding inquiry to discover possible resource configurations.
/// 2. **Reserve**: A preliminary commitment. The system guarantees resource availability
///    for a specific **Commit Timeout** period. Also is the Deletion of reservation
///    without additional cost is possible.
/// 3. **Commit**: The final handshake. Both the requester and the resource provider
///    fix the reservation parameters.
/// 4. **Delete**: Cancels a reservation. This is free during the "Reserved" state but
///    may impose penalties if the reservation was already "Committed."
pub trait ReservationProcessor {
    /// Sends a **Probe Request** to the resource management system.
    ///
    /// This is a read-only operation used to gather potential configurations for a
    /// reservation based on the system's current information-hiding policy.
    ///
    /// **Note**: A successful probe does not guarantee that a subsequent `reserve`
    /// call will succeed, though it is highly likely.
    ///
    /// # Arguments
    /// * `requester` - The component initiating the request, used for partner-based strategies.
    /// * `reservation` - The template reservation. Fields like `assigned_start` are ignored
    ///   in favor of `booking_interval`, `is_moldable` and `task_duration`.
    ///
    /// # Returns
    /// A `Reservations` object, of which all contained reservation are set to the state
    /// `ReservationState::ProbeAnswer`.
    fn probe(&self, requester: Box<dyn ReservationSubmitter>, reservation: Box<dyn Reservation>) -> Vec<Box<dyn Reservation>>;

    /// Sends a **Reserve Request** to initiate a preliminary commitment.
    ///
    /// The system will submit the task to the local Resource Management System (RMS)
    /// and hold the resources for a predefined timeout period. The requester must
    /// either `commit` or `delete` the reservation before this timeout expires.
    ///
    /// # Arguments
    /// * `requester` - The component initiating the request; also used for push-notifications
    ///   regarding reservation status changes.
    /// * `reservation` - The task details to reserve.
    ///
    /// # Returns
    /// A `Reservation` object. Success is indicated by `ReservationState::ReserveAnswer`.
    /// If resources cannot be held, returns `ReservationState::Rejected`.
    fn reserve(&self, requester: Box<dyn ReservationSubmitter>, reservation: Box<dyn Reservation>) -> Box<dyn Reservation>;

    /// Sends a **Commit Request** to finalize a reservation.
    ///
    /// This informs the local Resource Management System (RMS) that the task is
    /// formally committed. Once committed, the task is protected from deletion
    /// under normal operating conditions and may be subject to fines if canceled.
    ///
    /// **Note**: Shadow schedules cannot be committed at the individual task level.
    /// To apply changes from a shadow schedule, use [`Self::commit_shadow_schedule`].
    ///
    /// # Arguments
    /// * `requester` - The component that initiated the original reservation.
    /// * `reservation` - The task to commit. While the object may differ from the
    ///   original `reserve` call, the `task_name` must match.
    /// TODO Does the argument description still hold?
    ///
    /// # Returns
    /// A `Reservation` indicating the final status. Success is confirmed if the
    /// state is `ReservationState::Committed`.
    fn commit(&self, requester: Box<dyn ReservationSubmitter>, reservation: Box<dyn Reservation>) -> Box<dyn Reservation>;

    /// Sends a **Delete Request** to remove a task from the schedule.
    ///
    /// This removes a formerly submitted or reserved task from the local RMS.
    ///
    /// # Arguments
    /// * `requester` - The requesting component, used for partner-based strategies.
    /// * `reservation` - The task to be removed.
    ///
    /// # Returns
    /// A `Reservation` indicating the final status. Success is confirmed if
    /// the state is `ReservationState::Deleted`.
    fn delete(&self, requester: Box<dyn ReservationSubmitter>, reservation: Box<dyn Reservation>) -> Box<dyn Reservation>;
}
