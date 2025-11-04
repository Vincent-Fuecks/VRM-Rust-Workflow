/// Defines the lifecycle state of a job reservation within the system.
///
/// This state tracks the progress of the reservation from initial request
/// through processing, commitment, and eventual completion or failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// Specifies the process state the reservation is currently in. 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// A reservation represents any kind of job to be executed in the Grid. A 
/// reservation is used during all steps, so even if it is technically 
/// just a "reservation request" or if the reservation was rejected
/// 
/// The core data structure representing a resource reservation request.
///
/// This object is used throughout the entire lifecycle of a job, from the initial
/// request to final execution, and represents the current status of the
/// resource allocation.
///
/// Note that in a distributed setup, multiple objects may represent the same
/// reservation. They are identified by their unique `id` and rely on a
/// consistent synchronization across components.
#[derive(Debug, Clone)]
pub struct ReservationBase {
    // --- IDENTITY & STATE ---

    /// A unique identifier assigned to the reservation upon creation.
    pub id: int,

    /// The current state of this specific reservation instance.
    ///
    /// This may represent only the local state and might not perfectly
    /// reflect the global state of the reservation.
    pub state: ReservationState,

    /// The client's instruction on how far the reservation process should proceed.
    pub proceeding: ReservationProceeding,
    

    // --- TIME WINDOWS (All fields are in seconds) ---

    /// The time  this job arrived in the system.
    pub arrival_time: i32,

    /// The earliest possible start time for the job.
    pub booking_interval_start: i32,

    /// The latest possible end time for the job.
    pub booking_interval_end: i64,

    /// The scheduled start time of the job. Must be within the booking interval.
    pub assigned_start: i32,
    
    /// The scheduled end time of the job. Must be within the booking interval.
    pub assigned_end: i32,


    // --- RESOURCE & MOLDING ---

    /// Used for fragmentation calculation; a tolerance delta value.
    pub frag_delta: f32,
    
    /// The requested and reserved duration of the job (in seconds).
    pub job_duration: i32,

    /// The requested and reserved capacity of this job. 
    /// The capacity is measured in a unit according to the job type 
    /// e.g. number of CPUs for NodeReservation or kBit/s Bandwidth for LinkReservation 
    pub reserved_capacity: i32,
    
    /// If true, the `job_duration` and `reserved_capacity` are adjustable (moldable)
    /// during the reservation process to fit available resources.
    pub moldable: bool,

    /// Internal field: The total required work, calculated as (capacity * duration).
    /// Used internally to adjust capacity and duration while preserving the total work required.
    moldable_capacity: i32, 
}