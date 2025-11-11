use serde::{Deserialize, Serialize};

use crate::api::reservation_dto::{
    LinkReservationDto, NodeReservationDto, ReservationStateDto, ReservationProceedingDto
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDto {
    pub id: String,
    
    pub arrival_time: i64,
    pub booking_interval_start: i64,
    pub booking_interval_end: i64,
    
    pub tasks: Vec<TaskDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: String,
    pub reservation_state: ReservationStateDto, 
    pub request_proceeding: ReservationProceedingDto, 

    pub link_reservation: LinkReservationDto,
    pub node_reservation: NodeReservationDto,
}