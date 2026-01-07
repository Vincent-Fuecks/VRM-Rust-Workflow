use std::collections::HashMap;

use crate::api::workflow_dto::client_dto::{ClientDto, SystemModelDto};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{AdcId, ClientId, WorkflowId};
use crate::domain::vrm_system_model::workflow::workflow::Workflow;
use crate::error::Result;

/**
 * Client for the user to submit one or more jobs to the VRM
 * infrastructure. The client can be connected to an {@link ADC}
 * or directly to an {@link AI}.
 *
 * The client reads a config file containing one or more jobs.
 * These jobs are send to the {@link ADC} and automatically
 * committed, if not specified otherwise. Each job may have a
 * "arrival time" associated, the Client will wait then until
 * this time is reached before submitting it. This feature is
 * mainly for use in the simulation mode (see {@link Simulator}).
 *
 * All parameter are set in the {@link Reservation} object, also
 * how the client should handle (see {@link ReservationProceeding})
 * the reservation.
 *
 * @see ADC
 * @see AI
 */
/// Represents a client, which can have multiple workflows.
#[derive(Debug, Clone)]
pub struct Client {
    pub id: ClientId,
    pub workflows: HashMap<WorkflowId, Workflow>,

    unprocessed_reservations: Reservations,
    open_reservations: Reservations,
    adc_name: AdcId,
}

/// The root of the internal model, which can have multiple clients.
#[derive(Debug, Clone, Default)]
pub struct SystemModel {
    pub clients: HashMap<ClientId, Client>,
}

impl SystemModel {
    pub fn from_dto(root_dto: SystemModelDto) -> Result<Self> {
        let mut clients = HashMap::new();

        for client_dto in root_dto.clients {
            let client = Client::from_dto(client_dto)?;
            clients.insert(client.id.clone(), client);
        }
        Ok(SystemModel { clients })
    }
}

impl Client {
    pub fn from_dto(dto: ClientDto) -> Result<Self> {
        let mut workflows = HashMap::new();
        let client_id = ClientId::new(dto.id);

        for workflow_dto in dto.workflows {
            let workflow_id = WorkflowId::new(workflow_dto.id.clone());
            let workflow = Workflow::try_from((workflow_dto, client_id.clone()))?;
            workflows.insert(workflow_id, workflow);
        }

        let adc_name = todo!();
        let open_reservations = todo!();
        let unprocessed_reservations = todo!();
        Ok(Client { id: client_id, workflows, adc_name, open_reservations, unprocessed_reservations })
    }
}
