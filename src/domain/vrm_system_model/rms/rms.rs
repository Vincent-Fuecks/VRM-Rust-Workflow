use crate::api::rms_config_dto::rms_dto::DummyRmsDto;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::{Link, Node};
use crate::domain::vrm_system_model::scheduler_trait::Schedule;
use crate::domain::vrm_system_model::utils::id::{AciId, ResourceName, RmsId, RouterId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

use std::any::Any;

pub trait Rms: std::fmt::Debug + Any {
    fn get_base(&self) -> &RmsBase;
    fn get_base_mut(&mut self) -> &mut RmsBase;
    fn as_any(&self) -> &dyn Any;

    /// Performs the routing the correct scheduler
    ///
    /// Routs to the node_schedule or link_schedule based on the provided Reservation
    /// (LinkReservation or NodeReservation)
    /// of
    /// the master or shadowSchedule
    fn get_mut_active_schedule(&mut self, shadow_schedule_id: Option<ShadowScheduleId>, reservation_id: ReservationId) -> &mut Box<dyn Schedule>;

    fn set_reservation_state(&mut self, id: ReservationId, new_state: ReservationState) {
        self.get_base().reservation_store.update_state(id, new_state);
    }
}

#[derive(Debug)]
pub struct RmsBase {
    pub id: RmsId,
    pub resource_store: ResourceStore,
    pub reservation_store: ReservationStore,
}

#[derive(Debug)]
pub struct RmsLoadMetric {
    pub node_load_metric: Option<LoadMetric>,
    pub link_load_metric: Option<LoadMetric>,
}

impl RmsBase {
    pub fn new(aci_id: AciId, rms_type: String, reservation_store: ReservationStore, resource_store: ResourceStore) -> Self {
        let name = format!("AcI: {}, RmsType: {}", aci_id, &rms_type);

        if resource_store.get_num_of_nodes() <= 0 {
            log::info!("Empty Rms: The newly created Rms of type {} of AcI {} contains no Nodes", rms_type, aci_id);
        }

        RmsBase { id: RmsId::new(name), resource_store, reservation_store }
    }

    pub fn get_nodes_and_links(dto: &DummyRmsDto) -> (Vec<Node>, Vec<Link>) {
        let mut links = Vec::new();
        let mut nodes = Vec::new();

        for link_dto in &dto.network_links {
            let link = Link {
                id: ResourceName::new(link_dto.id.clone()),
                source: RouterId::new(link_dto.start_point.clone()),
                target: RouterId::new(link_dto.end_point.clone()),
                capacity: link_dto.capacity,
            };

            links.push(link);
        }

        for node_dto in &dto.grid_nodes {
            let node = Node {
                name: ResourceName::new(node_dto.id.clone()),
                cpus: node_dto.cpus,
                connected_to_router: node_dto.connected_to_router.iter().map(|router_id| RouterId::new(router_id)).collect(),
            };

            nodes.push(node);
        }

        return (nodes, links);
    }
}
