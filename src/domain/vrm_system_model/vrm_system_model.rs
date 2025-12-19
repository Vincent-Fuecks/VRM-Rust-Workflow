use std::collections::HashMap;

use crate::api::vrm_system_model_dto::vrm_dto::VrmSystemModelDto;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::adc::ADC;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};
use crate::error::ConversionError;
use std::sync::LazyLock;

const SIMULATOR: LazyLock<Box<dyn SystemSimulator>> = LazyLock::new(|| {
    let simulator = Simulator::new(true);
    Box::new(simulator)
});

#[derive(Debug)]
pub struct VrmSystemModel {
    adcs: HashMap<AdcId, ADC>,
    acis: HashMap<AciId, AcI>,
}

impl VrmSystemModel {
    pub fn from_dto(&mut self, root_dto: VrmSystemModelDto) -> Result<(), ConversionError> {
        for adc_dto in root_dto.adcs {
            let adc_id = AdcId::new(adc_dto.id.clone());
            let adc = ADC::try_from(adc_dto)?;
            self.adcs.insert(adc_id, adc);
        }

        for aci_dto in root_dto.acis {
            let aci_id = AciId::new(aci_dto.id.clone());
            let aci = AcI::try_from((aci_dto, SIMULATOR.clone()))?;
            self.acis.insert(aci_id, aci);
        }

        Ok(())
    }
}
