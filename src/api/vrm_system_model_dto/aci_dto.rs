use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcIDto {
    pub id: String,
    pub adc_ids: Vec<String>,
    pub commit_timeout: i64,
    pub rms_system: RMSSystemDto,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RMSSystemDto {
    pub typ: String,
    pub scheduler_type: String,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub grid_nodes: Vec<GridNodeDto>,
    pub network_links: Vec<NetworkLinkDto>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GridNodeDto {
    pub id: String,
    pub cpus: i64,
    pub connected_to_router: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkLinkDto {
    pub id: String,
    pub start_point: String,
    pub end_point: String,
    pub capacity: i64,
}
