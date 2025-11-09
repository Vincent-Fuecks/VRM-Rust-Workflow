use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use crate::workflow::reservation::ReservationState;

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
    pub name: String,
    // pub adc_id: String,
    
    /// --- TIME WINDOWS (All fields are in seconds) ---

    /// The time  this job arrived in the system.
    pub arrival_time: i64,

    /// The earliest possible start time for the job.
    pub booking_interval_start: i64,

    /// The latest possible end time for the job.
    pub booking_interval_end: i64,

    /// The scheduled start time of the job. Must be within the booking interval.
    pub assigned_start: i64,
    
    /// The scheduled end time of the job. Must be within the booking interval.
    pub assigned_end: i64,

    pub tasks: Vec<Task>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String, 
    pub name: String,
    pub state: TaskState, 
    
    // pub state: ReservationState,

    /// The client's instruction on how far the reservation process should proceed.
    pub request_proceeding: ReservationProceeding,

    /// TODO Is this redandent???
    // pub used_aci_hierarchy: Vec<String>,

    // --- RESOURCE & MOLDING ---

    /// Used for fragmentation calculation; a tolerance delta value.
    // #[serde(default = "min_f64")]
    // pub frag_delta: f64,
    
    // // /// The requested and reserved duration of the job (in seconds).
    // // #[serde(default = "min_i64")]
    // pub job_duration: i64,

    // /// The requested and reserved capacity of this job. 
    // /// The capacity is measured in a unit according to the job type 
    // /// e.g. number of CPUs for NodeReservation or kBit/s Bandwidth for LinkReservation 
    // pub reserved_capacity: i64,
    
    // /// If true, the `job_duration` and `reserved_capacity` are adjustable (moldable)
    // /// during the reservation process to fit available resources.
    // pub moldable: bool,

    // /// Internal field: The total required work, calculated as (`reserved_capacity` * `job_duration`).
    // ///
    // /// Used internally to adjust capacity and duration while preserving the total work required
    // /// for moldable reservations. 
    // moldable_capacity: i64, 


    pub link_reservation: LinkReservation,
    pub node_reservation: NodeReservation
}




#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataOut {
    pub name: String,
    pub file: Option<String>,
    pub size: Option<u64>,
    pub bandwidth: Option<u64>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataIn {
    pub source_reservation: String,
    pub source_port: String,
    pub file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dependency {
    pub pre: Vec<String>,
    pub sync: Vec<String>,
}
#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkReservation {
    pub start_point: String,
    pub end_point: String,
    pub amount: Option<u64>,
    pub bandwidth: Option<u64>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeReservation {
    pub task_path: Option<String>,
    pub output_path: Option<String>,
    pub error_path: Option<String>,
    pub duration: i64,
    pub cpus: i64, 
    pub is_moldable: bool, 
    pub dependencies: Dependency, 
    pub data_out: Vec<DataOut>, 
    pub data_in: Vec<DataIn>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TaskState {
    Probe,
    Commit,
    Open,
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