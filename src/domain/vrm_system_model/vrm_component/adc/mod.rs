mod actor;
mod handler;
mod helpers;
mod vrm_component;

use crate::{
    api::vrm_system_model_dto::adc_dto::ADCDto,
    domain::vrm_system_model::{
        vrm_component::utils::vrm_component_base::{VrmComponentBase, VrmComponentTyp},
        vrm_component::vrm_component_trait::VrmComponent,
    },
};
use std::collections::HashMap;

pub struct ADC {
    pub base: VrmComponentBase,
}

impl ADC {
    pub fn new(id: String) -> Self {
        let base = VrmComponentBase::new(id, HashMap::new(), None, VrmComponentTyp::ADC);

        Self { base }
    }

    pub fn new_vrm_component() -> Box<dyn VrmComponent> {
        todo!()
    }
}
