use std::collections::HashSet;

use crate::api::vrm_system_model_dto::aci_dto::{AcIDto, RMSSystemDto, RouterDto};
use crate::error::Error;

#[derive(Debug, Clone)]
pub enum ScheduleID {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

#[derive(Debug, Clone)]
pub struct AcI {
    id: String,
    adc_id: String,
    slot_width: i64,
    num_of_slots: i64,
    commit_timeout: i64,
    router_connections: Vec<String>,
    schedule_id: ScheduleID,
    rms_system: RMSSystem,
}

#[derive(Debug, Clone)]
pub struct RMSSystem {
    id: String,
    typ: String,
    routers: Vec<Router>,
    physical_nodes: Vec<PhysicalNode>,
}

#[derive(Debug, Clone)]
pub struct Router {
    id: String,
    typ: String,
    physical_links: Option<Vec<PhysicalLink>>,
}

#[derive(Debug, Clone)]
pub struct PhysicalLink {
    id: String,
    endpoint: String,
    capacity: i64,
}

#[derive(Debug, Clone)]
pub struct PhysicalNode {
    id: String,
    cpus: i64,
    connected_to_router: Vec<String>,
}

impl TryFrom<AcIDto> for AcI {
    type Error = Error;

    fn try_from(dto: AcIDto) -> Result<Self, Self::Error> {
        Ok(AcI {
            id: dto.id.clone(),
            adc_id: dto.adc_id,
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
            commit_timeout: dto.commit_timeout,
            router_connections: (),
            schedule_id: (),
            rms_system: (),
        })
    }
}

impl AcI {
    fn build_router_conections(dto: &RMSSystemDto) {
        let mut connected_to_routers: HashSet<String> = HashSet::new();

        for router in &dto.routers {
            if !connected_to_routers.contains(&router.id) {
                connected_to_routers.insert(router.id.clone());
            }
        }
    }

    fn build_rms_system(dto: &RMSSystemDto) {}
}
