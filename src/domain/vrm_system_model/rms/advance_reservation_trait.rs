use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::utils::id::ShadowScheduleId;
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

use std::cmp::Ordering;

/// Direct interface to a local Resource Management System (RMS) capable of making advance reservations.
///
/// This trait serves as the bridge between the high-level Virtual Resource Manager (VRM) and the
/// underlying local RMS. It facilitates the core reservation lifecycle: **Probe**, **Reserve**, and **Commit**.
///
/// # Core Functions:
/// 1.  **[probe](Self::probe)**: Queries the RMS for available reservation candidates based on constraints.
///     This operation is read-only and does not reserve resources.
/// 2.  **[reserve](Self::reserve)**: Temporarily reserves a job. This submits the request to the real RMS
///     but marks it as temporarily.
/// 3.  **[commit](Self::commit)**: Finalizes the reservation. Once committed, the reservation is fixed
///     and should not be cancelled via `delete_task` during normal operation.
/// 4.  **[delete_task](Self::delete_task)**: Cancels a reservation. This typically occurs after
///     `reserve` but before `commit`, or if the user explicitly cancels a job.
///
/// # Shadow Schedules
/// This interface supports **Shadow Schedules** isolated copies of the actual booking schedule.
/// Operations performed on a shadow schedule (identified by a [`ShadowScheduleId`]) do not affect
/// the live RMS until [commit_shadow_schedule](Self::commit_shadow_schedule) is called. This is critical
/// for distributed transactions and "what-if" planning phases in the Grid/VRM system.
pub trait AdvanceReservationRms: Rms {
    /// Creates a secondary **Shadow Schedule**.
    ///
    /// Initially, this schedule is an exact clone of the master schedule. It allows for
    /// manipulative operations (like testing reservations) without affecting the actual
    /// live reservations.
    ///
    /// # Arguments
    ///
    /// * `shadow_schedule_id` - A unique identifier for the new shadow schedule.
    ///
    /// # Errors
    ///
    /// Logs an error if a shadow schedule with the given ID already exists.
    fn create_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        if self.get_shadow_schedule_keys().contains(shadow_schedule_id) {
            log::error!(
                "Creating new shadow schedule is not possible because shadow schedule id ({}) does already exist. Please first delete the old shadow schedule.",
                shadow_schedule_id
            );
            return false;
        }

        let new_shadow_schedule = self.get_base_mut().schedule.clone_box();
        self.get_base_mut().shadow_schedules.insert(shadow_schedule_id.clone(), new_shadow_schedule);
        return true;
    }

    /// Commits a specific **Shadow Schedule**, replacing the master schedule.
    ///
    /// This operation applies all changes made in the simulation (shadow) phase to the live system.
    /// Since operations on the shadow schedule (like `reserve` or `delete_task`) validate constraints
    /// incrementally, the switch is generally expected to succeed.
    ///
    /// # Arguments
    ///
    /// * `shadow_schedule_id` - The shadow schedule with the provided identifier is promote to master.
    ///
    /// # Returns
    ///
    /// * `true` if the changes were successfully applied.
    /// * `false` if the shadow schedule could not be found, leaving the old master schedule valid.
    ///
    /// # Note
    ///
    /// After a successful commit, the `shadow_schedule_id` is consumed and no longer available.
    fn commit_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        let new_schedule = self.get_base_mut().shadow_schedules.remove(shadow_schedule_id);

        if new_schedule.is_some() {
            self.get_base_mut().schedule = new_schedule.unwrap();
            return true;
        }

        log::error!("Finding and removing shadow schedule with id {} was not possible", shadow_schedule_id.clone());
        return false;
    }

    /// Calculates the fragmentation of the schedule within a specific time range.
    ///
    /// Returns a value between `0.0` and `1.0`, where `0.0` represents optimal continuity
    /// (lowest fragmentation) and `1.0` represents high fragmentation.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the time window in VRM time (seconds).
    /// * `end` - The end of the time window in VRM time (seconds).
    /// * `shadow_schedule_id` - If `Some`, analyzes the specified shadow schedule.
    ///                          If `None`, analyzes the master schedule.
    fn get_fragmentation(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).get_fragmentation(start, end),
            None => self.get_mut_master_schedule().get_fragmentation(start, end),
        }
    }

    /// Calculates the global fragmentation of the entire schedule.
    ///
    /// # Arguments
    ///
    /// * `shadow_schedule_id` - If `Some`, analyzes the specified shadow schedule.
    ///                          If `None`, analyzes the master schedule.
    ///
    /// # Returns
    ///
    /// A value between `0.0` (best case) and `1.0` (worst case).
    fn get_system_fragmentation(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).get_system_fragmentation(),
            None => self.get_mut_master_schedule().get_system_fragmentation(),
        }
    }

    /// Retrieves load metrics for a specific time range.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the time window in VRM time (seconds).
    /// * `end` - The end of the time window in VRM time (seconds).
    /// * `shadow_schedule_id` - If `Some`, queries the specified shadow schedule.
    ///                          If `None`, queries the master schedule.
    ///
    /// # Returns
    ///
    /// A [`LoadMetric`] containing the calculated utilization metrics.
    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).get_load_metric_up_to_date(start, end),
            None => self.get_mut_master_schedule().get_load_metric_up_to_date(start, end),
        }
    }

    /// Retrieves load metrics for a specific time range.
    ///
    /// # Arguments
    ///
    /// * `start` - The start of the time window in VRM time (seconds).
    /// * `end` - The end of the time window in VRM time (seconds).
    /// * `shadow_schedule_id` - If `Some`, queries the specified shadow schedule.
    ///                          If `None`, queries the master schedule.
    ///
    /// # Returns
    ///
    /// A [`LoadMetric`] containing the calculated utilization metrics.
    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        match shadow_schedule_id {
            Some(id) => self.get_shadow_schedule(id).get_load_metric(start, end),
            None => self.get_master_schedule().get_load_metric(start, end),
        }
    }

    /// Retrieves load metrics for the total simulation.
    ///
    /// # Arguments
    ///
    /// * `shadow_schedule_id` - If `Some`, queries the specified shadow schedule.
    ///                          If `None`, queries the master schedule.
    ///
    /// # Returns
    ///
    /// A [`LoadMetric`] containing the calculated utilization metrics.
    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).get_simulation_load_metric(),
            None => self.get_mut_master_schedule().get_simulation_load_metric(),
        }
    }

    /// Probes the RMS for possible reservation configurations.
    ///
    /// This method checks the schedule for slots that satisfy the constraints of the given
    /// reservation request (e.g., task_duration, booking interval, is_moldable). It does **not** modify the schedule.
    ///
    /// # Arguments
    ///
    /// * `reservation_id` - The ID of the reservation to check. The check is based on
    ///   booking intervals and duration, ignoring currently assigned start/end times.
    /// * `shadow_schedule_id` - If `Some`, probes the specified shadow schedule.
    ///                          If `None`, probes the master schedule.
    ///
    /// # Returns
    ///
    /// A [`Reservations`] object containing a list of valid configuration candidates.
    /// Each candidate will have its state set to `ReservationState::ProbeAnswer`.
    /// If no candidates are found, an empty list is returned.
    /// TODO is the state of all reservation changed in the ReservationStore?
    fn probe(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> Reservations {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).probe(reservation_id),
            None => self.get_mut_master_schedule().probe(reservation_id),
        }
    }

    /// Submits a reservation request to the local RMS.
    ///
    /// This attempts to book the resource. If successful, the reservation is recorded
    /// in the schedule (shadow or master).
    ///
    /// # Arguments
    ///
    /// * `reservation_id` - The ID of the task to reserve.
    /// * `shadow_schedule_id` - If `Some`, reserves on the specified shadow schedule.
    ///                          If `None`, reserves on the master schedule.
    ///
    /// # Returns
    ///
    /// * `Some(ReservationId)` if the reservation was successful. The state will be
    ///   `ReservationState::ReserveAnswer`.
    /// * `None` if the reservation was rejected (e.g., due to conflicts). The state
    ///   will be `ReservationState::Rejected`
    fn reserve(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> Option<ReservationId> {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).reserve(reservation_id),
            None => self.get_mut_master_schedule().reserve(reservation_id),
        }
    }

    /// Finalizes a reservation, marking it as committed.
    ///
    /// This informs the RMS that the user has accepted the reservation and it is fixed.
    /// Committed jobs should not be deleted during normal operation.
    ///
    /// # Note on Implementation
    ///
    /// The default implementation logs the commit and updates the state to `ReservationState::Committed`.
    /// Implementors interfacing with hardware or external APIs should override this to propagate
    /// the commit signal to the physical RMS if necessary.
    ///
    /// # Arguments
    ///
    /// * `reservation_id` - The identifier of the task to commit.
    ///
    /// # Returns
    ///
    /// The `ReservationId` of the committed job.
    fn commit(&mut self, reservation_id: ReservationId) -> ReservationId {
        log::info!("RmsNull committed reservation with id: {:?}.  Please verify if specific RMS logic is required.", reservation_id);

        self.set_reservation_state(reservation_id, ReservationState::Committed);
        return reservation_id;
    }

    /// Destroys the specified **Shadow Schedule**.
    ///
    /// This is used to clean up simulation data. The master schedule remains active and unaffected.
    ///
    /// # Arguments
    ///
    /// * `shadow_schedule_id` - The unique identifier of the shadow schedule to remove.
    fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if self.get_base_mut().shadow_schedules.remove(&shadow_schedule_id).is_none() {
            log::error!("Removing shadow schedule was not possible. Shadow schedule id ({}) was not found", shadow_schedule_id);
            return false;
        }
        return true;
    }

    /// Probes for the single best reservation candidate based on a comparator.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the reservation request.
    /// * `comparator` - A closure defining the ordering logic to determine the "best" candidate.
    /// * `shadow_schedule_id` - If `Some`, probes the specified shadow schedule.
    ///                          If `None`, probes the master schedule.
    ///
    /// # Returns
    ///
    /// `Some(ReservationId)` of the best candidate, or `None` if no valid slots exist.
    fn probe_best(
        &mut self,
        request_id: ReservationId,
        comparator: &mut dyn FnMut(ReservationId, ReservationId) -> Ordering,
        shadow_schedule_id: Option<ShadowScheduleId>,
    ) -> Option<ReservationId> {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).probe_best(request_id, comparator),
            None => self.get_mut_master_schedule().probe_best(request_id, comparator),
        }
    }

    /// TODO Returned in java the ReservationId, If a failure occurred.
    /// Should not be necessary in the rust implementation.  
    /// Cancels and deletes a previously submitted reservation.
    ///
    /// This removes the reservation from the local schedule. It is primarily used during
    /// the negotiation phase (before `commit`) or if a user explicitly cancels a task.
    ///
    /// # Arguments
    ///
    /// * `reservation_id` - The ID of the job to delete.
    /// * `shadow_schedule_id` - If `Some`, deletes from the specified shadow schedule.   
    fn delete_task(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) {
        match shadow_schedule_id {
            Some(id) => self.get_mut_shadow_schedule(id).delete_reservation(reservation_id),
            None => self.get_mut_master_schedule().delete_reservation(reservation_id),
        }
    }

    fn can_handle_adc_request(&self, res: Reservation) -> bool {
        self.get_base().resources.can_handle_adc_request(res)
    }

    fn can_handle_aci_request(&self, reservation_store: ReservationStore, reservation_id: ReservationId) -> bool {
        self.get_base().resources.can_handle_aci_request(reservation_store, reservation_id)
    }

    fn get_total_link_capacity(&self) -> i64 {
        self.get_base().resources.get_total_link_capacity()
    }

    fn get_total_node_capacity(&self) -> i64 {
        self.get_base().resources.get_total_node_capacity()
    }

    fn get_total_capacity(&self) -> i64 {
        self.get_base().resources.get_total_capacity()
    }

    fn get_link_resource_count(&self) -> usize {
        self.get_base().resources.get_link_resource_count()
    }
}

impl<T: Rms> AdvanceReservationRms for T {}
