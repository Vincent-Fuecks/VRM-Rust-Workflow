use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::api::vrm_system_model_dto::adc_dto::ADCDto;

pub struct VrmSystemModelDto {
    pub adcs: Vec<ADCDto>,
    pub acis: Vec<AcIDto>,
}
