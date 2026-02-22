use std::any::Any;
use std::cmp::Ordering;
use std::fmt::Debug;

use crate::domain::vrm_system_model::reservation::probe_reservations::ProbeReservations;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

// TODO Sync is potentially unsafe; if total struct Sync than this should be redundant
pub trait Schedule: Debug + Send + Sync {
    /// Calculates the resource **fragmentation score** over a specific, user-defined time range.
    ///
    /// # Arguments
    ///
    /// * `frag_start_time` - The absolute start time for the fragmentation window (in seconds).
    /// * `frag_end_time` - The absolute end time for the fragmentation window (in seconds).
    ///
    /// # Returns
    ///
    /// A `f64` fragmentation score (typically **0.0** being best, **1.0** being worst).
    fn get_fragmentation(&mut self, frag_start_time: i64, frag_end_time: i64) -> f64;

    /// Calculates the resource **fragmentation score** across the **entire active scheduling window**.
    ///
    /// This method is intended to provide a system-wide view of allocated slots and often utilizes
    /// a cached value for performance if the schedule has not been modified since the last calculation.
    fn get_system_fragmentation(&mut self) -> f64;

    /// Retrieves resource **load metrics** (e.g., average reserved capacity, utilization)
    /// for a specified absolute time interval.
    ///
    /// # Arguments
    ///
    /// * `start_time` - The absolute start time of the query interval (in seconds).
    /// * `end_time` - The absolute end time of the query interval (in seconds).
    ///
    /// # Returns
    ///
    /// A `LoadMetrics` structure detailing the average capacity utilization and reserved capacity.
    fn get_load_metric_up_to_date(&mut self, start_time: i64, end_time: i64) -> LoadMetric;

    /// Retrieves resource **load metrics** (e.g., average reserved capacity, utilization)
    /// for a specified absolute time interval, which out an update.
    ///
    /// # Arguments
    ///
    /// * `start_time` - The absolute start time of the query interval (in seconds).
    /// * `end_time` - The absolute end time of the query interval (in seconds).
    ///
    /// # Returns
    ///
    /// A `LoadMetrics` structure detailing the average capacity utilization and reserved capacity.
    /// Note: The returned **Resource Load Metric** could be outed.
    fn get_load_metric(&self, start_time: i64, end_time: i64) -> LoadMetric;

    /// Retrieves load metrics for the **effective overall simulation period**.
    ///
    /// This period excludes initial and final slots defined by system configuration
    /// (`SLOTS_TO_DROP_ON_START`/`SLOTS_TO_DROP_ON_END`).
    fn get_simulation_load_metric(&mut self) -> LoadMetric;

    /// Performs a **feasibility probe** to find all possible time slots where a given reservation
    /// request can be accommodated.
    ///
    /// The probe returns a collection of candidate reservations, each representing a feasible time assignment.
    /// Additionally is also the system fragmentation impact of each candidate calculated and in
    /// `frag_delta` of the reservation stored.
    ///
    /// # Arguments
    ///
    /// * `id` - The `ReservationId` identifying the resource requirements and constraints for the probe.
    ///
    /// # Returns
    ///
    /// A `Reservations` collection containing all feasible candidates.
    fn probe(&mut self, id: ReservationId) -> ProbeReservations;

    /// Selects the **single best-fitting reservation candidate** from the feasible set,
    /// determined by a custom comparator function.
    ///
    /// This method first performs a standard `probe` to get all candidates, and then uses the provided
    /// comparator to select the optimal scheduling choice (e.g., earliest start time, least fragmentation impact).
    ///
    /// # Type Parameters
    ///
    /// * `C` - A closure that implements the `FnMut` trait for comparison, taking two reservation ids
    /// and returning an `Ordering`.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The `ReservationId` defining the request.
    /// * `comparator` - The function used to determine which candidate is "best."
    ///
    /// # Returns
    ///
    /// An `Option` containing the single `ReservationId` that best fits the criteria, or `None` if no feasible candidates were found.
    /// TODO Output must be changed to ProbeReservation, if System runs distributed --> because information of origen is lost, also deletion must be handled in a way.
    fn probe_best(
        &mut self,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
    ) -> Option<ReservationId>;

    /// Attempts to execute a **final reservation** using a provided candidate.
    ///
    /// If the attempt succeeds, the capacity is assigned, and `None` is returned. If capacity is
    /// unavailable, the reservation is marked as `Rejected` and returned inside `Some`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `ReservationId` candidate to finalize.
    ///
    /// # Returns
    ///
    /// `None` on success (reservation is accepted and committed), or `Some(ReservationId)` if the ReservationId is rejected.
    fn reserve(&mut self, id: ReservationId) -> Option<ReservationId>;

    /// **Commits a reservation** to the schedule **without performing a feasibility check**.
    ///
    /// This is an internal function typically called after a successful `probe` or by `reserve`
    /// to finalize the assignment. It assumes the reservation details are valid.
    ///
    /// # Arguments
    ///
    /// * `id` - The `ReservationId` to be inserted directly into the schedule slots.
    fn reserve_without_check(&mut self, id: ReservationId);

    /// Removes an **active reservation** from the schedule and frees up the occupied capacity
    /// in all relevant time slots.
    ///
    /// # Arguments
    ///
    /// * `Id` - The `ReservationId` of the reservation to be deleted.
    fn delete_reservation(&mut self, id: ReservationId);

    /// **Clears all active reservations** and resets the load of all slots to zero.
    fn clear(&mut self);

    /// **Updates the scheduling window** by advancing the internal time pointers based on the current simulation time.
    ///
    /// This process deletes all reservations that have expired (assigned end time is past the new start time)
    /// and moves the load from the now-expired slots into the `load_buffer` for historical tracking.
    fn update(&mut self);

    fn clone_box(&self) -> Box<dyn Schedule>;
}

impl Clone for Box<dyn Schedule> {
    fn clone(&self) -> Box<dyn Schedule> {
        self.clone_box()
    }
}
