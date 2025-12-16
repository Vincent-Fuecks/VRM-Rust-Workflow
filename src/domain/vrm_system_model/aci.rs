use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::rms::rms::Rms;
use crate::domain::vrm_system_model::rms::rms_type::RmsType;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};
use crate::error::ConversionError;

#[derive(Debug, Clone)]
pub enum ScheduleID {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

pub struct ReservationContainer {
    /// The ReservationSubmitter, which submitted the reservation.
    owner: ReservationSubmitter,
    ///  Until which time the reservation has to be committed, if only reserved. VRM time in s.
    commit_deadline: i64,
    execution_deadline: i64,
}
#[derive(Debug)]
pub struct AcI {
    pub id: AciId,
    adc_id: AdcId,
    commit_timeout: i64,
    rms_system: Box<dyn Rms>,
}

impl TryFrom<(AcIDto, Box<dyn SystemSimulator>)> for AcI {
    type Error = ConversionError;

    fn try_from(args: (AcIDto, Box<dyn SystemSimulator>)) -> Result<Self, ConversionError> {
        let (dto, simulator) = args;

        let aci_name = dto.id.clone();
        let rms_system = RmsType::get_instance(dto.rms_system, simulator, dto.id)?;

        Ok(AcI { id: AciId::new(aci_name), adc_id: AdcId::new(dto.adc_id), commit_timeout: dto.commit_timeout, rms_system })
    }
}
