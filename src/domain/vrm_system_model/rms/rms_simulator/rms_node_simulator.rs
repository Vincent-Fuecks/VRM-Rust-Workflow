use crate::api::rms_config_dto::rms_dto::DummyRmsDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase, RmsContext};
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::Node;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::id::{AciId, ResourceName, RouterId};
use crate::error::ConversionError;
use std::any::Any;
use std::str::FromStr;
use std::sync::Arc;

/// Only simulates a cluster with nodes (a Network with link reservations etc. is not managed)
#[derive(Debug)]
pub struct RmsNodeSimulator {
    pub base: RmsBase,
}

impl RmsNodeSimulator {
    pub fn new(base: RmsBase) -> Self {
        RmsNodeSimulator { base }
    }
}

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for RmsNodeSimulator {
    type Error = ConversionError;

    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args.clone();
        let resource_store = ResourceStore::new();

        let mut nodes = Vec::new();
        let mut schedule_capacity = 0;

        for node_dto in &dto.grid_nodes {
            let node = Node {
                name: ResourceName::new(node_dto.id.clone()),
                cpus: node_dto.cpus,
                connected_to_router: node_dto.connected_to_router.iter().map(|router_id| RouterId::new(router_id)).collect(),
            };

            schedule_capacity += node_dto.cpus;
            nodes.push(node);
        }

        let scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;

        let rms_context = RmsContext {
            aci_id: aci_id.clone(),
            rms_type: "NullRms".to_string(),
            schedule_capacity: schedule_capacity,
            slot_width: dto.slot_width,
            num_of_slots: dto.num_of_slots,
            nodes: nodes,
            reservation_store,
            simulator,
            schedule_type: scheduler_type,
        };

        let base = RmsBase::new(rms_context, resource_store.clone());

        Ok(RmsNodeSimulator { base })
    }
}

impl Rms for RmsNodeSimulator {
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

impl AdvanceReservationRms for RmsNodeSimulator {
    fn can_rms_handle_reservation(&self, reservation_id: ReservationId) -> bool {
        if self.get_base().reservation_store.is_node(reservation_id) {
            true
        } else {
            log::debug!(
                "The Reservation {:?} was submitted to the RmsNodeSimulator, which not of kind NodeReservation.",
                self.get_base().reservation_store.get_name_for_key(reservation_id)
            );
            self.get_base().reservation_store.update_state(reservation_id, ReservationState::Rejected);
            false
        }
    }
}
