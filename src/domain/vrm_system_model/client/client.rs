use crate::api::workflow_dto::client_dto::ClientsDto;
use crate::domain::vrm_system_model::reservation::reservation_store::{ReservationId, ReservationStore};
use crate::domain::vrm_system_model::utils::id::ClientId;
use crate::domain::vrm_system_model::workflow::workflow::Workflow;
use crate::error::Result;

#[derive(Debug)]
pub struct Clients {
    pub unprocessed_reservations: Vec<ReservationId>,
}

impl Clients {
    pub fn from_dto(dto: ClientsDto, reservation_store: ReservationStore) -> Result<Self> {
        let mut unprocessed = Vec::new();

        for client_dto in dto.clients {
            let client_id = ClientId::new(client_dto.id);

            for workflow_dto in client_dto.workflows {
                let workflow_res_id = Workflow::create_form_dto(workflow_dto, client_id.clone(), reservation_store.clone())?;
                unprocessed.push(workflow_res_id);
            }
        }

        Ok(Clients { unprocessed_reservations: unprocessed })
    }
}
