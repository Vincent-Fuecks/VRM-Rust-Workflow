use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::api::vrm_system_model_dto::adc_dto::ADCDto;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VrmDto {
    pub adc: Vec<ADCDto>,
    pub aci: Vec<AcIDto>,
}
