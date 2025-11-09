use serde::{Deserialize, Serialize};
use crate::domain::reservation::ReservationProceeding;
use uuid::Uuid;
use crate::api::workflow_dto::{
    TaskDto, WorkflowDto, LinkReservationDto, NodeReservationDto,
    DependencyDto, DataOutDto, DataInDto
};
use log::warn;

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

/// TODO Add Comment + TaskStates are not right!!! 
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TaskState {
    Probe,
    Commit,
    Open,
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

impl From<WorkflowDto> for Workflow {
    fn from(dto: WorkflowDto) -> Self {
        if dto.booking_interval_start > dto.booking_interval_end {
            warn!(
                "Workflow '{}' has booking_interval_start ({}) after booking_interval_end ({}).", 
                dto.name, dto.booking_interval_start, dto.booking_interval_end
            );
        }

        Self {
            name: dto.name,
            arrival_time: dto.arrival_time,
            booking_interval_start: dto.booking_interval_start,
            booking_interval_end: dto.booking_interval_end,
            assigned_start: i64::MIN,
            assigned_end: i64::MIN,   

            tasks: dto.tasks.into_iter().map(Task::from).collect(),
        }
    }
}

impl From<TaskDto> for Task {
    fn from(dto: TaskDto) -> Self {
        Self {
            id: "TODO".to_string(), 
            name: dto.name,
            state: dto.state,
            request_proceeding: dto.request_proceeding,
            
            link_reservation: LinkReservation::from(dto.link_reservation),
            node_reservation: NodeReservation::from(dto.node_reservation),
        }
    }
}

impl From<LinkReservationDto> for LinkReservation {
    fn from(dto: LinkReservationDto) -> Self {
        Self {
            start_point: dto.start_point,
            end_point: dto.end_point,
            amount: dto.amount,
            bandwidth: dto.bandwidth,
        }
    }
}

impl From<NodeReservationDto> for NodeReservation {
    fn from(dto: NodeReservationDto) -> Self {
        Self {
            task_path: dto.task_path,
            output_path: dto.output_path,
            error_path: dto.error_path,
            duration: dto.duration,
            cpus: dto.cpus,
            is_moldable: dto.is_moldable,
            dependencies: Dependency::from(dto.dependencies),
            data_out: dto.data_out.into_iter().map(DataOut::from).collect(),
            data_in: dto.data_in.into_iter().map(DataIn::from).collect(),
        }
    }
}

impl From<DependencyDto> for Dependency {
    fn from(dto: DependencyDto) -> Self {
        Self {
            pre: dto.pre, 
            sync: dto.sync 
        }
    }
}

impl From<DataOutDto> for DataOut {
    fn from(dto: DataOutDto) -> Self {
        Self {name: dto.name, 
            file: dto.file, 
            size: dto.size, 
            bandwidth: dto.bandwidth
        }
    }
}

impl From<DataInDto> for DataIn {
    fn from(dto: DataInDto) -> Self {
        Self {
            source_reservation: dto.source_reservation, 
            source_port: dto.source_port, 
            file: dto.file
        }
    }
}