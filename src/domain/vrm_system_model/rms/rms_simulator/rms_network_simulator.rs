use crate::api::rms_config_dto::rms_dto::DummyRmsDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase, RmsContext};
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::{Link, NetworkTopology, Node};
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::id::{AciId, ResourceName, RouterId};
use crate::error::ConversionError;
use std::any::Any;
use std::str::FromStr;
use std::sync::Arc;

/// Only simulates a cluster with Links (Nodes are not simulated)
#[derive(Debug)]
pub struct RmsNetworkSimulator {
    pub base: RmsBase,
}

impl Rms for RmsNetworkSimulator {
    fn get_base(&self) -> &RmsBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut RmsBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for RmsNetworkSimulator {
    type Error = ConversionError;

    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args;
        let (nodes, links) = RmsNetworkSimulator::get_nodes_and_links(&dto);
        let resource_store = ResourceStore::new();

        // Adds Links to Resource Store
        let topology = NetworkTopology::new(
            &links,
            &nodes,
            dto.slot_width,
            dto.num_of_slots,
            simulator.clone(),
            aci_id.clone(),
            reservation_store.clone(),
            resource_store.clone(),
        );

        let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
        scheduler_type = scheduler_type.get_network_scheduler_variant(topology, resource_store.clone());

        let rms_context = RmsContext {
            aci_id: aci_id.clone(),
            rms_type: "NullBroker".to_string(),
            schedule_capacity: i64::MAX,
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
            nodes: nodes,
            reservation_store,
            simulator,
            schedule_type: scheduler_type,
        };

        // Adds Nodes to Resource Store
        let base = RmsBase::new(rms_context, resource_store.clone());

        Ok(RmsNetworkSimulator { base: base })
    }
}

impl RmsNetworkSimulator {
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

impl AdvanceReservationRms for RmsNetworkSimulator {
    fn can_rms_handle_reservation(&self, reservation_id: ReservationId) -> bool {
        if self.get_base().reservation_store.is_link(reservation_id) {
            true
        } else {
            log::debug!(
                "The Reservation {:?} was submitted to the RmsNodeSimulator, which not of kind LinkReservation.",
                self.get_base().reservation_store.get_name_for_key(reservation_id)
            );
            self.get_base().reservation_store.update_state(reservation_id, ReservationState::Rejected);
            false
        }
    }
}
