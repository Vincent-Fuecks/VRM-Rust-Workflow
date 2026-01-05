use std::collections::HashMap;

use crate::api::vrm_system_model_dto::vrm_dto::VrmSystemModelDto;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::adc::ADC;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};
use crate::error::Result;
use std::sync::LazyLock;

const SIMULATOR: LazyLock<Box<dyn SystemSimulator>> = LazyLock::new(|| {
    let simulator = Simulator::new(true);
    Box::new(simulator)
});

#[derive(Debug)]
pub struct VrmSystemModel {
    pub adcs: HashMap<AdcId, ADC>,
    pub acis: HashMap<AciId, AcI>,
}

impl VrmSystemModel {
    pub fn from_dto(root_dto: VrmSystemModelDto) -> Result<Self> {
        let mut vrm_system_model = VrmSystemModel { adcs: HashMap::new(), acis: HashMap::new() };

        for adc_dto in root_dto.adc {
            let adc_id = AdcId::new(adc_dto.id.clone());
            let adc = ADC::try_from(adc_dto)?;
            vrm_system_model.adcs.insert(adc_id, adc);
        }

        for aci_dto in root_dto.aci {
            let aci_id = AciId::new(aci_dto.id.clone());
            let aci = AcI::try_from((aci_dto, SIMULATOR.clone()))?;
            vrm_system_model.acis.insert(aci_id, aci);
        }

        Ok(vrm_system_model)
    }
}
