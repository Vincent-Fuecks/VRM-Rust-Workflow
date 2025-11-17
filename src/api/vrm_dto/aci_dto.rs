use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcIDto {
    id: String,
    adc_id: String,
    slot_width: i64,
    num_of_slots: i64,
    commit_timeout: i64,
    connected_to_routers: Vec<String>,
    scheduler_typ: String,
    rms_system: RMSSystemDto,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RMSSystemDto {
    id: String,
    typ: String,
    routers: Vec<Router>,
    physical_nodes: Vec<PhysicalNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Router {
    id: String,
    typ: String,
    physical_links: Option<Vec<PhysicalLink>>,
}

#[derive(Debug, Deserialize)]
pub struct PhysicalLink {
    id: String,
    endpoint: String,
    capacity: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalNode {
    id: String,
    cpus: i64,
    connected_to_router: Vec<String>,
}
