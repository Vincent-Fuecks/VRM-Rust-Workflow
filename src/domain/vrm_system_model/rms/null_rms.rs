use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::error::ConversionError;
use std::any::Any;
use std::ops::Not;

#[derive(Debug)]
pub struct NullRms {
    pub base: RmsBase,
}

impl NullRms {
    pub fn new(base: RmsBase) -> Self {
        NullRms { base }
    }
}

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String)> for NullRms {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String)) -> Result<Self, Self::Error> {
        let base = RmsBase::try_from(args)?;
        if base.grid_nodes.is_empty() {
            log::info!("Empty NullRms Grid: The newly created NullRms contains no Gird Nodes.");
        }
        if base.network_links.is_empty().not() {
            log::info!("Not Empty NullRms Network: NullRms should not contain a Network. NullBroker or Slurm should be utilized instead.");
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
