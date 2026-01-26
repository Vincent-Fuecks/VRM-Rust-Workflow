use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::domain::vrm_system_model::utils::id::AciId;
use crate::error::ConversionError;
use std::any::Any;
use std::sync::Arc;

#[derive(Debug)]
pub struct NullRms {
    pub base: RmsBase,
}

impl NullRms {
    pub fn new(base: RmsBase) -> Self {
        NullRms { base }
    }
}

impl TryFrom<(RMSSystemDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for NullRms {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
        let (_, _, aci_id, _) = args.clone();
        let base = RmsBase::try_from(args)?;
        if base.resources.get_node_resource_count() == 0 {
            log::info!("Empty NullRms Grid: The newly created NullRms of AcI {} contains no Gird Nodes.", aci_id);
        }

        if base.resources.get_link_resource_count() > 0 {
            log::info!(
                "Not Empty NullRms Link Network: The newly created NullRms of AcI {} contains links. These are ignored by the NullRms do you like to use NullBroker or Slurm as Rms system?",
                aci_id
            );
        }

        Ok(NullRms { base })
    }
}

impl Rms for NullRms {
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
