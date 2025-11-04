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

    /// Internal field: The total required work, calculated as (`reserved_capacity` * `job_duration`).
    ///
    /// Used internally to adjust capacity and duration while preserving the total work required
    /// for moldable reservations.
    moldable_capacity: i32, 
}


impl ReservationBase {
    /// Recalculates and updates the internal `moldable_capacity` based on the current
    /// `reserved_capacity` and `job_duration`.
    fn update_moldable_capacity(&mut self) {
        self.moldable_capacity = self.reserved_capacity * self.job_duration;
    }


    /// Sets a new job duration and recalculates the internal `moldable_capacity` based on this change.
    ///
    /// This method performs an **inherent change** in the job size (total work). To adjust the
    /// duration while keeping the total work constant for moldable jobs, use [`adjust_job_duration`].
    pub fn set_job_duration(&mut self, job_duration: i32) {
        self.job_duration = job_duration;
        self.moldable_capacity = self.moldable_capacity * self.job_duration;
    }


    /// Adjusts the job duration and recalculates the requested capacity for **moldable reservations**.
    ///
    /// This operation changes the duration and capacity such that the total required work
    /// (`moldable_capacity`) remains constant. For inherent changes to the job size, use [`set_job_duration`].
    pub fn adjust_job_duration(&mut self, duration: i32) {
        if !self.moldable {
            // TODO Logging --> Logger.error("adjustJobDuration for non moldable job " + this);
            eprintln!("Warning: adjust_job_duration called for non-moldable job. Proceeding with adjustment.");
        }

        self.job_duration = duration.max(1);
        self.reserved_capacity = (self.moldable_capacity / self.job_duration).max(1);
    }

    /// Adjusts the reserved capacity and recalculates the job duration for **moldable reservations**.
    ///
    /// This operation changes the capacity and duration such that the total required work
    /// (`moldable_capacity`) remains constant. For inherent changes to the job size, use
    /// [`set_reserved_capacity`].
    ///
    /// The capacity unit is specific to the job type (e.g., CPUs or Bandwidth).
    pub fn adjust_capacity(&mut self, capacity: i32) {
        if capacity != self.reserved_capacity {
            if !self.moldable {
                // TODO Logging --> Logger.error("adjustCapacity for non moldable job " + this);
                eprintln!("Warning: adjust_capacity called for non-moldable job. Proceeding with adjustment.");
            }

            self.reserved_capacity = capacity.max(1);
            self.job_duration = (self.moldable_capacity / self.reserved_capacity).max(1);
        }
    }

    /// Sets a new reserved capacity and recalculates the internal `moldable_capacity` based on this change.
    ///
    /// This method performs an **inherent change** in the job size (total work). To adjust the
    /// capacity while keeping the total work constant for moldable jobs, use  [`adjust_capacity`].
    ///
    /// The capacity unit is specific to the job type (e.g., CPUs or Bandwidth).
    pub fn set_reserved_capacity(&mut self, reserved_capacity: i32) {
        self.reserved_capacity = reserved_capacity;
        self.moldable_capacity = self.reserved_capacity * self.job_duration;
    }

    /// Determines if two reservations are considered logically equal based on their unique ID.
    ///
    /// Two reservations are equal if they carry the same unique ID (`id`). If both reservations
    /// have no ID assigned (`None`), they are only considered equal if they reference the
    /// exact same object instance.
    pub fn equal_name(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }

        match (&self.id, &other.id) {
            (Some(name1), Some(name2)) => name1 == name2,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (None, None) => false,
        }
    }
}