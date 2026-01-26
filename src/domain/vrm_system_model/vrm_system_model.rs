use std::collections::HashMap;
use std::sync::Arc;

use crate::api::vrm_system_model_dto::vrm_dto::VrmDto;
use crate::domain::simulator;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::adc::ADC;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, ComponentId};
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Vrm {
    pub adc_master: AdcId,
    pub adcs: HashMap<AdcId, ADC>,
    pub acis: HashMap<AciId, AcI>,
}

// impl Vrm {
//     pub fn from_dto(root_dto: VrmDto, simulator: Arc<dyn SystemSimulator>) -> Result<Self> {
//         let mut vrm_system_model = Vrm { adc_master: AdcId::new(root_dto.adc_master), adcs: HashMap::new(), acis: HashMap::new() };

//         for adc_dto in root_dto.adc {
//             let adc_id = AdcId::new(adc_dto.id.clone());
//             let adc = ADC::try_from(adc_dto)?;

//             vrm_system_model.adcs.insert(adc_id, adc);
//         }

//         for aci_dto in root_dto.aci {
//             let aci_id = AciId::new(aci_dto.id.clone());
//             let aci = AcI::try_from((aci_dto, simulator.clone_box().into()))?;
//             vrm_system_model.acis.insert(aci_id, aci);
//         }

//         Ok(vrm_system_model)
//     }

//     pub fn add_clients() -> bool {
//         todo!()boxed_aci
//     }
// }

// impl TryFrom<(VrmDto, ReservationStore)> for Vrm {
//     type Error = Error;

//     fn try_from((dto, reservation_store): (VrmDto, ReservationStore)) -> Result<Vrm> {
//         let component_master_id = ComponentId::new(dto.adc_master);
//         let mut components = HashMap::new();
//         for adc_dto in dto.adc {
//             let component_id = ComponentId::new(adc_dto.id);
//         }

//         todo!()
//     }
// }
