use crate::error::Error;
use crate::{api::vrm_system_model_dto::aci_dto::GridNodeDto, domain::vrm_system_model::reservation::reservation::ReservationKey};

#[derive(Debug, Clone)]
pub struct GridNode {
    pub id: ReservationKey,
    pub cpus: String,
    pub connected_to_router: Vec<ReservationKey>,
}

impl TryFrom<GridNodeDto> for GridNode {
    type Error = Error;

    fn try_from(dto: GridNodeDto) -> Result<Self, Self::Error> {
        let mut connected_to_router = Vec::new();

        for router_name in dto.connected_to_router {
            connected_to_router.push(ReservationKey { id: router_name });
        }

        Ok(GridNode { id: ReservationKey { id: dto.id }, cpus: dto.cpus, connected_to_router: connected_to_router })
    }
}
