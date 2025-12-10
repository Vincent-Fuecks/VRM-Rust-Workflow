use std::collections::HashSet;

use crate::api::vrm_system_model_dto::aci_dto::{AcIDto, RMSSystemDto};
use crate::domain::vrm_system_model::reservation::reservation::ReservationKey;
use crate::domain::vrm_system_model::scheduler_type::SchedulerType;
use crate::error::Error;

#[derive(Debug, Clone)]
pub enum ScheduleID {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

#[derive(Debug, Clone)]
pub struct AcI {
    pub id: ReservationKey,
    adc_id: ReservationKey,
    commit_timeout: i64,
    rms_system: RMSSystem,
}

impl TryFrom<AcIDto> for AcI {
    type Error = Error;

    fn try_from(dto: AcIDto) -> Result<Self, Self::Error> {
        let rms_system = None;

        Ok(AcI {
            id: ReservationKey { id: dto.id.clone() },
            adc_id: ReservationKey { id: dto.adc_id },
            commit_timeout: dto.commit_timeout,
            rms_system: todo!(),
        })
    }
}
