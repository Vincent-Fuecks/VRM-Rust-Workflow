use serde::{Deserialize};

use crate::api::workflow_dto::WorkflowDto;

#[derive(Debug, Deserialize)]
pub struct SystemModelDto {
    pub clients: Vec<ClientDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDto {
    pub id: String,
    pub workflows: Vec<WorkflowDto>,
}