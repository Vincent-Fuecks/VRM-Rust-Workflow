use crate::api::vrm_dto::aci_dto::AcIDto;
use crate::api::vrm_dto::adc_dto::ADCDto;

struct VRMDto {
    adcs: Vec<ADCDto>,
    acis: Vec<AcIDto>,
}
