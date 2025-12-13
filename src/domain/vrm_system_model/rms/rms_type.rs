use crate::api::vrm_system_model_dto::aci_dto::RMSSystemDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::rms::{null_broker::NullBroker, null_rms::NullRms, rms::Rms};
use crate::error::ConversionError;
use std::str::FromStr;

#[derive(Debug)]
pub enum RmsType {
    NullRms,
    NullBroker,
    Slurm,
}

impl RmsType {
    pub fn get_instance(dto: RMSSystemDto, simulator: Box<dyn SystemSimulator>, aci_name: String) -> Result<Box<dyn Rms>, ConversionError> {
        let rms_type: RmsType = RmsType::from_str(&dto.typ)?;

        match rms_type {
            RmsType::NullRms => {
                let rms_instance = NullRms::try_from((dto, simulator, aci_name))?;
                Ok(Box::new(rms_instance))
            }

            RmsType::NullBroker => {
                let broker_instance = NullBroker::try_from((dto, simulator, aci_name))?;
                Ok(Box::new(broker_instance))
            }
            RmsType::Slurm => {
                todo!()
            }
        }
    }
}

impl FromStr for RmsType {
    type Err = ConversionError;

    fn from_str(rms_type_dto: &str) -> Result<RmsType, Self::Err> {
        match rms_type_dto {
            "nullRms" => Ok(RmsType::NullRms),
            "NullBroker" => Ok(RmsType::NullBroker),
            "Slurm" => Ok(RmsType::Slurm),
            _ => Err(ConversionError::UnknownRmsType(rms_type_dto.to_string())),
        }
    }
}
