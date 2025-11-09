use serde::{Deserialize, Serialize};

/// Defines the lifecycle state of a job reservation within the system.
///
/// This state tracks the progress of the reservation from initial request
/// through processing, commitment, and eventual completion or failure.
/// 
/// The order, from lowest commitment (0) to highest (6), is:
/// 1.  `Rejected`
/// 2.  `Deleted`
/// 3.  `Open`
/// 4.  `ProbeAnswer`
/// 5.  `ReserveAnswer`
/// 6.  `Committed`
/// 7.  `Finished`
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReservationState {
    /// The last request of the reservation was explicitly denied or failed.
    Rejected,
    /// The reservation has been successfully cancelled and removed from the system.
    Deleted,
    /// The reservation is newly created and has not yet been submitted to any processor.
    Open,
    /// The state represents a successful response to a probing (availability) request.
    ProbeAnswer,
    /// The state represents a successful response to a resource reservation request.
    ReserveAnswer,
    /// The reservation has been confirmed and resources are formally allocated.
    Committed,
    /// The execution phase of the job linked to this reservation has been finished successfully.
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationProceeding {
    /// Only perform the initial **probe** request to check availability.
    Probe,
    /// Send only a reserve request and quit then. Do not cancel the reservation.
    Reserve,
    /// Commit the reservation
    Commit,
    /// Reserve the reservation, but delete it within the commit timeout
    Delete,
}