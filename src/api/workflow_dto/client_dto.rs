use serde::Deserialize;

use crate::api::workflow_dto::workflow_dto::WorkflowDto;

#[derive(Debug, Deserialize)]
pub struct ClientsDto {
    pub clients: Vec<ClientDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDto {
    pub id: String,
    pub adc_id: Option<String>,
    pub workflows: Vec<WorkflowDto>,
}
