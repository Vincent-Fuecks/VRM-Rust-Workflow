use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::aci;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::domain::vrm_system_model::schedule::topology::NetworkTopology;

use crate::domain::vrm_system_model::utils::id::AciId;
use crate::error::ConversionError;
use std::any::Any;
use std::sync::Arc;

#[derive(Debug)]
pub struct NullBroker {
    pub base: RmsBase,
    pub network_topology: NetworkTopology,
}

impl Rms for NullBroker {
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

impl TryFrom<(RMSSystemDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for NullBroker {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args;
        let base = RmsBase::try_from((dto.clone(), simulator.clone(), aci_id.clone(), reservation_store.clone()))?;
        let network_topology = NetworkTopology::try_from((dto, simulator, aci_id.clone(), reservation_store.clone()))?;

        if base.resources.get_node_resource_count() <= 0 {
            log::info!("Empty NullBroker Grid: The newly created NullBroker of AcI {} contains no Gird Nodes.", aci_id);
        }

        // If network is empty check is later in setup_network_links in topology.rs done.

        Ok(NullBroker { base, network_topology: network_topology })
    }
}
