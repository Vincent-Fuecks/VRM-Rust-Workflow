use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::domain::vrm_system_model::schedule::topology::NetworkTopology;

use crate::error::ConversionError;
use std::any::Any;

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

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String, ReservationStore)> for NullBroker {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_name, reservation_store) = args;
        let base = RmsBase::try_from((dto.clone(), simulator.clone(), aci_name.clone(), reservation_store))?;
        let network_topology = NetworkTopology::try_from((dto, simulator, aci_name))?;

        if base.resources.get_node_resource_count() <= 0 {
            log::info!("Empty NullBroker Grid: The newly created NullBroker contains no Gird Nodes.");
        }

        if base.resources.get_link_resource_count() <= 0 {
            log::info!("Empty NullBroker Link Network: The newly created NullBroker contains no LinkNetwork. Please use NullRms instead.");
        }

        Ok(NullBroker { base, network_topology: network_topology })
    }
}
