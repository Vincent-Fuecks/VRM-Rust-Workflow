use crate::api::rms_config_dto::rms_dto::{DummyRmsDto, RmsSystemWrapper};
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::{null_broker::NullBroker, null_rms::NullRms};
use crate::domain::vrm_system_model::utils::id::AciId;
use crate::error::ConversionError;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub enum RmsDummyType {
    NullRms,
    NullBroker,
}

impl RmsSystemWrapper {
    pub fn get_instance(
        dto: RmsSystemWrapper,
        simulator: Arc<dyn SystemSimulator>,
        aci_id: AciId,
        reservation_store: ReservationStore,
    ) -> Result<Box<dyn AdvanceReservationRms>, ConversionError> {
        match dto {
            RmsSystemWrapper::Slurm(data) => {
                todo!()
            }

            RmsSystemWrapper::DummyRms(dummy_rms_dto) => {
                let rms_type = RmsDummyType::from_str(&dummy_rms_dto.typ)?;

                match rms_type {
                    RmsDummyType::NullRms => {
                        let rms_instance = NullRms::try_from((dummy_rms_dto, simulator, aci_id, reservation_store))?;
                        Ok(Box::new(rms_instance))
                    }

                    RmsDummyType::NullBroker => {
                        let broker_instance = NullBroker::try_from((dummy_rms_dto, simulator, aci_id, reservation_store))?;
                        Ok(Box::new(broker_instance))
                    }
                }
            }
        }
    }
}

impl FromStr for RmsDummyType {
    type Err = ConversionError;

    fn from_str(rms_type_dto: &str) -> Result<RmsDummyType, Self::Err> {
        match rms_type_dto {
            "NullRms" => Ok(RmsDummyType::NullRms),
            "NullBroker" => Ok(RmsDummyType::NullBroker),
            _ => Err(ConversionError::UnknownRmsType(rms_type_dto.to_string())),
        }
    }
}
