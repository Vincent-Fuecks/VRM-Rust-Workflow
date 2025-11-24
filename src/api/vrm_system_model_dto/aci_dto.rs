use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcIDto {
    pub id: String,
    pub adc_id: String,
    pub slot_width: i64,
    pub num_of_slots: i64,
    pub commit_timeout: i64,
    pub connected_to_routers: Vec<String>,
    pub scheduler_typ: String,
    pub rms_system: RMSSystemDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RMSSystemDto {
    pub id: String,
    pub typ: String,
    pub routers: Vec<RouterDto>,
    pub physical_nodes: Vec<NodeResourceDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterDto {
    pub id: String,
    pub typ: String,
    pub physical_links: Option<Vec<LinkResourceDto>>,
}

#[derive(Debug, Deserialize)]
pub struct LinkResourceDto {
    pub id: String,
    pub endpoint: String,
    pub capacity: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeResourceDto {
    pub id: String,
    pub cpus: i64,
    pub connected_to_router: Vec<String>,
}
