use std::any::Any;

pub trait Reservation: std::fmt::Debug + Any {
    fn is_moldable(&self) -> bool;
    fn get_reserved_capacity(&self) -> i64;
    fn get_id(&self) -> String;
    fn get_assigned_start(&self) -> i64;
    fn get_assigned_end(&self) -> i64;

    fn set_state(&mut self, reservation_sate: ReservationState);
    fn set_assigned_start(&mut self, time: i64);
    fn set_assigned_end(&mut self, time: i64);

    /// Downcasting to NodeReservation/LinkReservation
    fn as_any(&self) -> &dyn Any;
}

impl Reservation for NodeReservation {
    fn get_id(&self) -> String {
        self.base.id
    }

    fn get_assigned_end(&self) -> i64 {
        self.base.assigned_end
    }

    fn get_assigned_start(&self) -> i64 {
        self.base.assigned_start
    }

    fn is_moldable(&self) -> bool {
        self.base.is_moldable
    }

    fn get_reserved_capacity(&self) -> i64 {
        self.base.reserved_capacity
    }

    fn set_assigned_end(&mut self, time: i64) {
        self.base.assigned_end = time;
    }

    fn set_assigned_start(&mut self, time: i64) {
        self.base.assigned_start = time;
    }
    fn set_state(&mut self, reservation_sate: ReservationState) {
        self.base.state = reservation_sate;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Reservation for LinkReservation {
    fn get_id(&self) -> String {
        self.base.id
    }

    fn get_assigned_end(&self) -> i64 {
        self.base.assigned_end
    }

    fn get_assigned_start(&self) -> i64 {
        self.base.assigned_start
    }

    fn is_moldable(&self) -> bool {
        self.base.is_moldable
    }

    fn get_reserved_capacity(&self) -> i64 {
        self.base.reserved_capacity
    }

    fn set_assigned_end(&mut self, time: i64) {
        self.base.assigned_end = time;
    }

    fn set_assigned_start(&mut self, time: i64) {
        self.base.assigned_start = time;
    }
    fn set_state(&mut self, reservation_sate: ReservationState) {
        self.base.state = reservation_sate;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ReservationKey {
    pub id: String,
}

/// Defines the lifecycle state of a job reservation within the system.
///
/// This state tracks the progress of the reservation from initial request
/// through processing, commitment, and eventual completion or failure.
///
/// The order, from lowest commitment (0) to highest (6).
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

/// Defines the set of primary actions (proceedings) that can be requested for a reservation.
///
/// This determines the lifecycle stage a reservation is intended to reach.
/// TODO Rework states transition discription
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservationProceeding {
    /// Executes only the initial resource availability **probe** request to check feasibility.
    /// No resources are formally reserved.
    Probe,

    /// Executes a **reserve** request to temporarily hold resources. The reservation
    /// remains active but requires a subsequent `Commit` or `Delete` action.
    Reserve,

    /// Executes the full sequence: **reserve** followed immediately by a **commit** to finalize
    /// the resource allocation.
    Commit,

    /// Reserve the reservation, but delete it within the commit timeout
    Delete,
}

/// The fundamental structure holding common data for any resource reservation system.
///
/// This base provides essential metadata, time information, and capacity requirements,
/// regardless of whether the reservation targets a computational node, network link or workflow.
///
/// It is also possible, that more than one object representing the same reservation.
/// Each component has to take care to sync the state of all these objects,
/// e.g., between the object in the reservation database and the return value of some request.
///
/// To support this synchronization each reservation has to carry a unique id within the VRM setup.
/// The component initially creating the reservation has to take care to generate only unique names.
/// Objects representing the same reservation, but in different states
/// or different configurations have to carry the very same id.
#[derive(Debug, Clone)]
pub struct ReservationBase {
    /// A globally unique identifier for this reservation across all distributed components.
    pub id: String,

    /// The current **lifecycle state** of the reservation (e.g., `Open`, `Committed`, `Finished`).
    pub state: ReservationState,

    /// The **requested action** for the reservation (e.g., `Probe`, `Reserve`, `Commit`).
    pub request_proceeding: ReservationProceeding,

    // Time windows in s
    /// The time at which the reservation request **arrived** in the system.
    pub arrival_time: i64,

    /// The earliest time the resource is **requested to start** the booking interval.
    pub booking_interval_start: i64,

    /// The latest time the resource is **requested to end** the booking interval.
    pub booking_interval_end: i64,

    /// The precise time the resource was **formally assigned to start** the task. (0 if not set).
    pub assigned_start: i64,

    /// The precise time the resource was **formally assigned to end** the task. (0 if not set).
    pub assigned_end: i64,

    // Resource properties
    /// The requested and reserved **duration** of the task (in seconds).
    pub task_duration: i64, // 'duration' in NodeReservationDto

    /// The total **amount of resource capacity** requested and reserved of this task
    /// Unit is according to the Task:
    /// NodeReservation: Number of CPUs
    /// LinkReservation: Bandwidth in Mbps
    pub reserved_capacity: i64,

    // Flag indicating if the task's duration and capacity are **flexible** (moldable/ can be adjusted).
    pub is_moldable: bool,

    /// Internal field: The total required work, calculated as `reserved_capacity` * `task_duration`.
    /// This value remains constant for non-moldable jobs.
    pub moldable_work: i64,
    // TODO
    // pub frag_delta: f64,
}

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

/// This structure extends [`ReservationBase`] to include fields specific to
/// network connectivity.
///
/// Link reservations typically have two use cases:
/// 1. **Data Transfer:** Reserving bandwidth for file transfer between two sites.
///    In this case, the reservation may be **moldable**, meaning the duration
///    can be adjusted based on available bandwidth.
/// 2. **Co-allocated Communication:** Reserving a specific, fixed amount of
///    bandwidth for short-term coordination and communication between tasks
///    associated with co-allocated compute reservations. The specified bandwidth
///    **must** be provided for the entire duration.
#[derive(Debug, Clone)]
pub struct LinkReservation {
    /// The common base properties shared by all reservations.
    pub base: ReservationBase,

    // Link specific fields
    /// Unique identifier of the start router for the link.
    pub start_point: String,
    /// Unique identifier of the end router for the link.
    pub end_point: String,
}
