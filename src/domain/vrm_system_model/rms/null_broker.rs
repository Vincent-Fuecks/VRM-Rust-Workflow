use crate::api::rms_config_dto::rms_dto::{DummyRmsDto, NetworkLinkDto};
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::domain::vrm_system_model::schedule::slotted_schedule::network_slotted_schedule::topology::NetworkTopology;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::domain::vrm_system_model::utils::id::AciId;
use crate::error::ConversionError;
use std::any::Any;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct NullBroker {
    pub base: RmsBase,
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

impl TryFrom<(DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for NullBroker {
    type Error = ConversionError;

    fn try_from(args: (DummyRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (dto, simulator, aci_id, reservation_store) = args;

        let topology = NetworkTopology::try_from((dto.clone(), simulator.clone(), aci_id.clone(), reservation_store.clone()))?;

        let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
        scheduler_type = scheduler_type.get_network_scheduler_variant(topology);

        let base = RmsBase::try_from((dto, simulator.clone(), aci_id.clone(), reservation_store, scheduler_type))?;

        if base.resources.get_node_resource_count() <= 0 {
            log::info!("Empty NullBroker Grid: The newly created NullBroker of AcI {} contains no Gird Nodes.", aci_id);
        }

        // If network is empty check is later in setup_network_links in topology.rs done.

        Ok(NullBroker { base })
    }
}


