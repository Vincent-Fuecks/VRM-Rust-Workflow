use std::collections::HashMap;

use crate::api::vrm_system_model_dto::vrm_dto::VrmSystemModelDto;
use crate::domain::vrm_system_model::aci::AcI;
use crate::domain::vrm_system_model::adc::ADC;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct VrmSystemModel {
    adcs: HashMap<String, ADC>,
    acis: HashMap<String, AcI>,
}

impl VrmSystemModel {
    pub fn from_dto(root_dto: VrmSystemModelDto) -> Result<Self> {
        let mut adcs = HashMap::new();
        let mut acis = HashMap::new();

        for adc_dto in root_dto.adcs {
            let adc = ADC::try_from(adc_dto)?;
            adcs.insert(adc.id.clone(), adc);
        }

        for aci_dto in root_dto.acis {
            let aci = AcI::from_dto(aci_dto)?;
            acis.insert(aci.id.clone(), aci);
        }

        Ok(VrmSystemModel {
            adcs: adcs,
            acis: acis,
        })
    }
}
