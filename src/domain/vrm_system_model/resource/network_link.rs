use crate::error::Error;
use crate::{api::vrm_system_model_dto::aci_dto::NetworkLinkDto, domain::vrm_system_model::reservation::reservation::ReservationKey};

#[derive(Debug, Clone)]
pub struct NetworkLink {
    pub id: ReservationKey,
    pub start_point: ReservationKey,
    pub end_point: ReservationKey,
    pub capacity: i64,
}

impl TryFrom<NetworkLinkDto> for NetworkLink {
    type Error = Error;

    fn try_from(dto: NetworkLinkDto) -> Result<Self, Self::Error> {
        Ok(NetworkLink {
            id: ReservationKey { id: dto.id },
            start_point: ReservationKey { id: dto.start_point },
            end_point: ReservationKey { id: dto.end_point },
            capacity: dto.capacity,
        })
    }
}
