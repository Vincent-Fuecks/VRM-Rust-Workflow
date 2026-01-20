use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, sleep};

use crate::api::workflow_dto::client_dto::{ClientDto, ClientsDto};
use crate::domain::simulator;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::client;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationTrait};
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservations::Reservations;
use crate::domain::vrm_system_model::utils::id::{AdcId, ClientId, WorkflowId};
use crate::domain::vrm_system_model::vrm_system_model::Vrm;
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
    pub simulator: Arc<dyn SystemSimulator>,

    pub unprocessed_reservations: Vec<Reservation>,
    pub open_reservations: Vec<Reservation>,
    pub adc_id: Option<AdcId>,
}

#[derive(Debug)]
pub struct Clients {
    pub clients: HashMap<ClientId, Client>,
}

impl Clients {
    pub fn from_dto(dto: ClientsDto, simulator: Arc<dyn SystemSimulator>, reservation_store: ReservationStore) -> Result<Self> {
        let mut clients = HashMap::new();

        for client_dto in dto.clients {
            let client_id = ClientId::new(client_dto.id);
            let mut unprocessed = Vec::new();

            for workflow_dto in client_dto.workflows {
                let workflow = Workflow::create_form_dto(workflow_dto, client_id.clone(), reservation_store.clone())?;
                unprocessed.push(Reservation::Workflow(workflow));
            }

            let adc_id = client_dto.adc_id.map(AdcId::new);

            let client = Client {
                id: client_id.clone(),
                simulator: simulator.clone(),
                adc_id,
                open_reservations: Vec::new(),
                unprocessed_reservations: unprocessed,
            };

            clients.insert(client_id, client);
        }

        Ok(Clients { clients })
    }
}

impl Client {
    pub async fn run(mut self, reservation_store: ReservationStore, adc: Arc<tokio::sync::Mutex<Box<dyn VrmComponent>>>) {
        self.unprocessed_reservations.sort_by_key(|r| r.get_arrival_time());

        while !self.unprocessed_reservations.is_empty() {
            let reservation = self.unprocessed_reservations.remove(0);

            let now = self.simulator.get_current_time_in_s();
            let arrival = reservation.get_arrival_time();

            if arrival > now {
                let wait_seconds = arrival - now;
                if wait_seconds > 0 {
                    sleep(Duration::from_secs(wait_seconds as u64)).await;
                }
            }

            let res_id = reservation_store.add(reservation);

            // Process the reservation (Probe -> Reserve -> Commit)
            // self.process_reservation(res_id, &reservation_store, adc).await;
        }

        log::info!("Client {} finished processing all reservations.", self.id);
    }

    async fn process_reservation(&mut self, reservation_id: ReservationId, reservation_store: &ReservationStore, adc: &mut Box<dyn VrmComponent>) {
        todo!()
    }
}
