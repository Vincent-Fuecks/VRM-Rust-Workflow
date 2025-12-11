use std::collections::HashMap;

use crate::api::vrm_system_model_dto::vrm_dto::VrmSystemModelDto;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::aci::AcI;
use crate::domain::vrm_system_model::adc::ADC;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::error::ConversionError;
use std::sync::LazyLock;

#[derive(Debug)]
pub struct VrmSystemModel {
    adcs: HashMap<ReservationKey, ADC>,
    acis: HashMap<ReservationKey, AcI>,
}
const SIMULATOR: LazyLock<Box<dyn SystemSimulator>> = LazyLock::new(|| {
    let simulator = Simulator::new(true);
    Box::new(simulator)
});
impl VrmSystemModel {
    pub fn from_dto(&self, root_dto: VrmSystemModelDto) -> Result<Self, ConversionError> {
        let mut adcs = HashMap::new();
        let mut acis = HashMap::new();

        for adc_dto in root_dto.adcs {
            let adc = ADC::try_from(adc_dto);
            adcs.insert(ReservationKey::new(adc_dto.id.clone()), adc);
        }

        for aci_dto in root_dto.acis {
            let aci = AcI::try_from((aci_dto, SIMULATOR.clone()))?;
            acis.insert(ReservationKey::new(aci_dto.id.clone()), aci);
        }

        Ok(VrmSystemModel { adcs: adcs, acis: acis })
    }
}
