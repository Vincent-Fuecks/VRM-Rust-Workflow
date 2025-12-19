use std::{any::Any, ops::Not};

use crate::domain::vrm_system_model::utils::id::{ClientId, ComponentId, ReservationName};

pub trait Reservation: std::fmt::Debug + Any + Send + Sync {
    fn get_base(&self) -> &ReservationBase;

    fn get_base_mut(&mut self) -> &mut ReservationBase;

    fn box_clone(&self) -> Box<dyn Reservation>;

    fn as_any(&self) -> &dyn Any;

    fn get_name(&self) -> ReservationName {
        self.get_base().name.clone()
    }

    fn get_assigned_start(&self) -> i64 {
        self.get_base().assigned_start
    }

    fn get_assigned_end(&self) -> i64 {
        self.get_base().assigned_end
    }

    fn is_moldable(&self) -> bool {
        self.get_base().is_moldable
    }

    fn get_reserved_capacity(&self) -> i64 {
        self.get_base().reserved_capacity
    }

    fn get_task_duration(&self) -> i64 {
        self.get_base().task_duration
    }

    fn get_state(&self) -> ReservationState {
        self.get_base().state
    }

    fn get_booking_interval_start(&self) -> i64 {
        self.get_base().booking_interval_start
    }

    fn get_booking_interval_end(&self) -> i64 {
        self.get_base().booking_interval_end
    }

    fn get_moldable_work(&self) -> i64 {
        self.get_base().moldable_work
    }

    fn get_client_id(&self) -> ClientId {
        self.get_base().client_id.clone()
    }

    fn get_handler_id(&self) -> Option<ComponentId> {
        self.get_base().handler_id.clone()
    }

    fn set_assigned_end(&mut self, time: i64) {
        self.get_base_mut().assigned_end = time;
    }

    fn set_assigned_start(&mut self, time: i64) {
        self.get_base_mut().assigned_start = time;
    }

    fn set_state(&mut self, reservation_state: ReservationState) {
        self.get_base_mut().state = reservation_state;
    }

    fn set_task_duration(&mut self, duration: i64) {
        self.get_base_mut().task_duration = duration;
    }

    fn set_reserved_capacity(&mut self, reserved_capacity: i64) {
        self.get_base_mut().reserved_capacity = reserved_capacity;
    }

    fn set_booking_interval_start(&mut self, start_time: i64) {
        self.get_base_mut().booking_interval_start = start_time;
    }

    fn set_booking_interval_end(&mut self, end_time: i64) {
        self.get_base_mut().booking_interval_end = end_time;
    }

    fn set_frag_delta(&mut self, frag_delta: f64) {
        self.get_base_mut().frag_delta = frag_delta;
    }

    fn adjust_capacity(&mut self, capacity: i64) {
        if capacity != self.get_base().reserved_capacity {
            if self.is_moldable().not() {
                log::warn!("adjustCapacity for non moldable job {}", self.get_base().get_name(),);
            }

            if capacity == 0 {
                self.set_task_duration(self.get_moldable_work());
                self.set_reserved_capacity(1);
            } else {
                self.set_task_duration(self.get_moldable_work() / capacity);
                self.set_reserved_capacity(capacity);
            }

            if self.get_task_duration() <= 0 {
                self.set_task_duration(1);
            }
        }
    }
}

impl Clone for Box<dyn Reservation> {
    fn clone(&self) -> Box<dyn Reservation> {
        self.box_clone()
    }
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
/// TODO Rework states transition description
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
    pub name: ReservationName,

    /// Is the Id of the client, how submitted the reservation into the VRM system.
    pub client_id: ClientId,

    /// Contains the Id of the components, how is handling the reservation currently (Adc or AcI).
    pub handler_id: Option<ComponentId>,

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
    /// LinkReservation: Bandwidth in MB's
    pub reserved_capacity: i64,

    // Flag indicating if the task's duration and capacity are **flexible** (moldable/ can be adjusted).
    pub is_moldable: bool,

    /// Internal field: The total required work, calculated as `reserved_capacity` * `task_duration`.
    /// This value remains constant for non-moldable jobs.
    pub moldable_work: i64,
    // TODO
    pub frag_delta: f64,
}

impl ReservationBase {
    pub fn is_moldable(&self) -> bool {
        self.is_moldable
    }

    pub fn get_reserved_capacity(&self) -> i64 {
        self.reserved_capacity
    }

    pub fn get_assigned_start(&self) -> i64 {
        self.assigned_start
    }

    pub fn get_assigned_end(&self) -> i64 {
        self.assigned_end
    }

    pub fn get_name(&self) -> ReservationName {
        self.name.clone()
    }

    pub fn set_state(&mut self, state: ReservationState) {
        self.state = state;
    }
}
