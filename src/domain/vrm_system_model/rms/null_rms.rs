use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::resource::grid_node::GridNode;
use crate::domain::vrm_system_model::resource::network_link::NetworkLink;
use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::error::ConversionError;
use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;

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
