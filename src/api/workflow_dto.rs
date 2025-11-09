use serde::{Deserialize, Serialize};
use crate::domain::workflow::{TaskState};
use crate::domain::reservation::ReservationProceeding;

#[derive(Debug, Deserialize)]
pub struct RootDto {
    pub clients: Vec<ClientDto>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct ClientDto {
    pub id: String,
    pub workflows: Vec<WorkflowDto>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowDto {
    pub name: String,
    
    /// --- TIME WINDOWS ---
    pub arrival_time: i64,
    pub booking_interval_start: i64,
    pub booking_interval_end: i64,
    
    pub tasks: Vec<TaskDto>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskDto {
    pub name: String,
    pub state: TaskState, 
    pub request_proceeding: ReservationProceeding, 

    pub link_reservation: LinkReservationDto,
    pub node_reservation: NodeReservationDto,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkReservationDto {
    pub start_point: String,
    pub end_point: String,
    pub amount: Option<u64>,
    pub bandwidth: Option<u64>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeReservationDto {
    pub task_path: Option<String>,
    pub output_path: Option<String>,
    pub error_path: Option<String>,
    pub duration: i64,
    pub cpus: i64,
    pub is_moldable: bool,
    pub dependencies: DependencyDto,
    pub data_out: Vec<DataOutDto>,
    pub data_in: Vec<DataInDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DependencyDto {
    pub pre: Vec<String>,
    pub sync: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataOutDto {
    pub name: String,
    pub file: Option<String>,
    pub size: Option<u64>,
    pub bandwidth: Option<u64>,
}

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataInDto {
    pub source_reservation: String,
    pub source_port: String,
    pub file: Option<String>,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct ReservationDto {
    /// --- IDENTITY & STATE ---
    pub id: Option<String>,
    pub proceeding: ReservationProceeding,
    pub used_aci_hierarchy: Vec<String>,
    
    /// --- TIME WINDOWS (All fields are in seconds) ---
    pub arrival_time: i64,
    pub booking_interval_start: i64,
    pub booking_interval_end: i64,
    pub assigned_start: i64,
    pub assigned_end: i64,

    // --- RESOURCE & MOLDING ---
    pub frag_delta: f32,
    pub job_duration: i32,
    pub reserved_capacity: i32,
    pub moldable: bool,
}