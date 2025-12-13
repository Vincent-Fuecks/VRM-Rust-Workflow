use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::error::ConversionError;
use std::any::Any;

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

impl TryFrom<(RMSSystemDto, Box<dyn SystemSimulator>, String)> for NullBroker {
    type Error = ConversionError;

    fn try_from(args: (RMSSystemDto, Box<dyn SystemSimulator>, String)) -> Result<Self, Self::Error> {
        let base = RmsBase::try_from(args)?;

        if base.grid_nodes.is_empty() {
            log::info!("Empty NullBroker Grid: The newly created NullBroker contains no Gird Nodes.");
        }

        if base.network_links.is_empty() {
            log::info!("Empty NullBroker Network: The newly created NullBroker contains no Network. NullRms should be utilized instead.");
        }

        Ok(NullBroker { base })
    }
}
