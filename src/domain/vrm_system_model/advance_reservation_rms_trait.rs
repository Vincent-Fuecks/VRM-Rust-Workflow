use std::cmp::Ordering;

use crate::domain::vrm_system_model::resource::resources::Resources;
use crate::domain::workflow::reservation::Reservation;

/// This is a trait for a local Resource Management System (RMS) capable of making advance reservations.
///
/// This trait defines the abstraction layer between the Virtual Resource Manager (VRM) and the
/// underlying local scheduler. It handles the lifecycle of a reservation, from probing availability
/// to committing a finalized schedule.
///
/// # Core Concepts
/// * **Probing:** Checking for availability without locking resources ([`Self::probe_best`]).
/// * **Reserving:** Temporary booking a task on the RMS ([`Self::reserve`]).
/// * **Committing:** Reserve resources for task ([`Self::commit`]).
/// * **Shadow Schedules:** Is a mechanism, which allows the system to simulate changes
///     on a copy of the schedule before applying them to the real RMS.
///     This allows for "what-if" scenarios without side effects.
///
/// # Shadow Schedule Lifecycle
/// 1.  Create a shadow copy using [`Self::create_shadow_schedule`].
/// 2.  Perform operations (reserve/delete) passing the `shadow_schedule_id`.
/// 3.  If the resulting schedule is valid, call [`Self::commit_shadow_schedule`].
/// 4.  If invalid or optimization fails, call [`Self::rollback_shadow_schedule`].
/// TODO Rework States in Comments, if trait is for one RMS implemented
pub trait AdvanceReservationRMS {
    /// Calculates the fragmentation of the schedule within a specific time range.
    ///
    /// Returns a metric indicating how fragmented the free space is. This is used by schedulers
    /// to avoid placing tasks in a way that creates unusable gaps (checkerboarding).
    ///
    /// # Arguments
    /// * `start` - The beginning of the time window to analyze (VRM Time in seconds).
    /// * `end` - The end of the time window to analyze (VRM Time in seconds).
    /// * `shadow_schedule_id` - The ID of the shadow schedule to analyze. If `None`, the
    ///   analysis is performed on the active (live) schedule.
    ///
    /// # Returns
    /// * `f64` - A value between 0.0 and 1.0, where:
    ///     * `0.0`: Lowest fragmentation (Best case).
    ///     * `1.0`: Highest fragmentation (Worst case).
    fn get_fragmentation(&self, start: i64, end: i64, shadow_schedule_id: Option<&str>) -> f64;

    /// Calculates the fragmentation of the entire schedule.
    ///
    /// Unlike [`Self::get_fragmentation`], this method considers the global state of the system
    /// rather than a specific window.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - The ID of the shadow schedule to analyze. If `None`, the
    ///   analysis is performed on the active (live) schedule.
    ///
    /// # Returns
    /// * `f64` - A value between 0.0 and 1.0, where:
    ///     * `0.0`: Lowest fragmentation (Best case).
    ///     * `1.0`: Highest fragmentation (Worst case).
    fn get_shadow_schedule_system_fragmentation(&self, shadow_schedule_id: Option<&str>) -> f64;

    // TODO First implement Loader class
    // fn get_load(&self, start: i64, end: i64, shadow_schedule_id: Option<&str>) -> LoadStatus;
    // fn get_simulation_load(&self) -> LoadStatus;

    /// Creates a new shadow schedule.
    ///
    /// Initially, the shadow schedule is an exact copy of the current normal schedule.
    /// It can be manipulated via `reserve` and `delete_task` without affecting actual reservations
    /// until committed.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - A unique identifier for the new shadow schedule.
    fn create_shadow_schedule(&mut self, shadow_schedule_id: String);

    /// Submits a shadow schedule into the normal schedule.
    ///
    /// This replaces all reservations in the normal schedule with those from the shadow schedule.
    /// Since operations on the shadow schedule are pre-validated via the RMS, this operation
    /// is generally expected to succeed.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - The ID of the shadow schedule to commit.
    ///
    /// # Returns
    /// * `true` - If the changes were successfully applied. The `shadow_schedule_id` becomes invalid.
    /// * `false` - If the commit failed. The previous normal schedule remains active.
    fn commit_shadow_schedule(&mut self, shadow_schedule_id: &str) -> bool;

    /// Discards a shadow schedule and reverts any virtual changes.
    ///
    /// The normal schedule remains active and untouched.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - The ID of the shadow schedule to delete.
    fn rollback_shadow_schedule(&mut self, shadow_schedule_id: &str);

    // TODO First implement Reservations class
    /// Identifies all possible reservation candidates that hold the provided contrains.
    ///
    /// This method searches for possible reservation slots that satisfy the constraints
    /// of the provided `reservation` (ignoring assigned start/end, using booking interval and duration).
    /// It compares valid candidates using the provided `comparator` closure and returns the best match.
    ///
    /// # Arguments
    /// * `reservation` - The template reservation containing constraints (duration, moldability, booking window).
    /// * `comparator` - A closure defining the optimization criteria (e.g., earliest start time, best fit).
    ///
    /// # Returns
    /// * `Option<Reservation>` - The best available reservation candidate in the `STATE_PROBEANSWER` state,
    ///   or `None` if no valid slot was found.
    // fn probe(&self, res: &Reservation, shadow_schedule_id: Option<&str>) -> Reservations;

    /// Identifies the optimal reservation candidate based on a provided comparator.
    ///
    /// This method searches for possible reservation slots that satisfy the constraints
    /// of the provided `reservation` (ignoring assigned start/end, using booking interval and duration).
    /// It compares valid candidates using the provided `comparator` closure and returns the best match.
    ///
    /// # Arguments
    /// * `reservation` - The template reservation containing constraints (duration, moldability, booking window).
    /// * `comparator` - A closure defining the optimization criteria (e.g., earliest start time, best fit).
    ///
    /// # Returns
    /// * `Option<Reservation>` - The best available reservation candidate in the `STATE_PROBEANSWER` state,
    ///   or `None` if no valid slot was found.
    fn probe_best(
        &self,
        reservation: &Reservation,
        comparator: &dyn Fn(&Reservation, &Reservation) -> Ordering,
    ) -> Option<Reservation>;

    /// Submits a task reservation to the local RMS.
    ///
    /// Attempts to book the task. If `shadow_schedule_id` is provided, the booking is virtual.
    /// If `None`, the booking is attempted on the real system.
    ///
    /// # Arguments
    ///
    /// * `res` - The task to reserve. The specific assigned start/end times in this object may be ignored
    ///   in favor of the RMS finding a valid slot within the booking interval, depending on implementation.
    /// * `shadow_schedule_id` - The shadow schedule to apply this reservation to, or `None` for the normal schedule.
    ///
    /// # Returns
    ///
    /// * `Option<Reservation>` - A reservation object with updated details (including assigned times).
    ///   State will be `STATE_RESERVEANSWER` on success or `STATE_REJECTED` on failure.
    fn reserve(
        &mut self,
        res: Reservation,
        shadow_schedule_id: Option<&str>,
    ) -> Option<Reservation>;

    /// Cancels a previously submitted task in the local RMS.
    ///
    /// This should typically be called after `reserve` but before `commit`. It may also be used
    /// if an end-user explicitly cancels a task.
    ///
    /// # Arguments
    /// * `res` - The task to delete. Must match the task `id` of a reserved task.
    /// * `shadow_schedule_id` - The shadow schedule to apply this deletion to, or `None` for the normal schedule.
    ///
    /// # Returns
    /// * `Option<Reservation>` - The updated reservation object. State will be `DELETED` on success
    ///   or `STATE_REJECTED` if the task could not be found or deleted.
    fn delete_task(
        &mut self,
        res: &Reservation,
        shadow_schedule_id: Option<&str>,
    ) -> Option<Reservation>;

    /// Finalizes a reservation, marking it as committed.
    ///
    /// Informs the RMS that the user has accepted the reservation. After this point,
    /// the task is considered fixed and cannot be deleted during normal scheduling optimization cycles.
    ///
    /// **Note:** Shadow schedules cannot be committed via this method; use [`Self::commit_shadow_schedule`] instead.
    ///
    /// # Arguments
    /// * `res` - The task to commit. Matches by task `id`.
    ///
    /// # Returns
    /// * `Option<Reservation>` - The updated reservation object. State will be `COMMITED` on success.
    fn commit(&mut self, res: Reservation) -> Option<Reservation>;

    /// Retrieves the definitions of resources managed by this RMS.
    ///
    /// # Returns
    /// * `&Resources` - A reference to the resource topology/list. Usually contains a single item,
    ///   but may represent a cluster or multi-system setup.
    fn get_resources(&self) -> &Resources;

    // TODO Do I need this?
    // fn generate_statistic_event(&self) -> StatisticEvent;
}
