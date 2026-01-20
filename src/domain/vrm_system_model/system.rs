use std::collections::HashMap;
use std::sync::Arc;

use crate::api::vrm_system_model_dto::vrm_dto::VrmDto;
use crate::api::workflow_dto::client_dto::{ClientDto, ClientsDto};
use crate::domain::simulator;
use crate::domain::simulator::simulator::{Simulator, SystemSimulator};
use crate::domain::vrm_system_model::adc::ADC;
use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use crate::domain::vrm_system_model::reservation::reservation_store::{self, ReservationStore};
use crate::domain::vrm_system_model::{
    client::client::Client,
    utils::id::{AciId, AdcId, ClientId, WorkflowId},
    vrm_system_model::Vrm,
    workflow::workflow::Workflow,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct System {
    pub clients: HashMap<ClientId, Client>,
    pub vrm: Vrm,
}
impl System {
    pub fn new(clients: HashMap<ClientId, Client>, vrm: Vrm) -> Self {
        System { clients, vrm }
    }
}

impl System {
    fn from_clients_dto(
        dto: ClientsDto,
        simulator: Arc<dyn SystemSimulator>,
        reservation_store: ReservationStore,
    ) -> Result<HashMap<ClientId, Client>> {
        let clients = HashMap::new();

        for client_dto in dto.clients {
            let mut workflows = HashMap::new();
            let client_id = ClientId::new(client_dto.id);

            for workflow_dto in client_dto.workflows {
                let workflow_id = WorkflowId::new(workflow_dto.id.clone());
                let workflow = Workflow::create_form_dto(workflow_dto, client_id.clone(), reservation_store.clone())?;
                workflows.insert(workflow_id, workflow);
            }

            let adc_id;
            if let Some(id) = client_dto.adc_id {
                adc_id = Some(AdcId::new(id));
            } else {
                adc_id = None;
            }

            let open_reservations = todo!();
            let unprocessed_reservations = todo!();

            let client = Client { id: client_id, simulator, adc_id, open_reservations, unprocessed_reservations };

            clients.insert(client_id, client);
        }

        Ok(clients)
    }

    fn from_vrm_dto(dto: VrmDto, simulator: Arc<dyn SystemSimulator>) -> Result<(HashMap<AdcId, ADC>, HashMap<AciId, AcI>)> {
        let mut adcs: HashMap<AdcId, ADC> = HashMap::new();
        let mut acis: HashMap<AciId, AcI> = HashMap::new();

        for adc_dto in dto.adc {
            let adc_id = AdcId::new(adc_dto.id.clone());
            let adc = ADC::try_from(adc_dto)?;
            adcs.insert(adc_id, adc);
        }

        for aci_dto in dto.aci {
            let aci_id = AciId::new(aci_dto.id.clone());
            let aci = AcI::try_from((aci_dto, simulator.clone()))?;
            acis.insert(aci_id, aci);
        }

        Ok((adcs, acis))
    }

    // pub async fn run_all_clients(&mut self, reservation_store: ReservationStore, simulator: Arc<dyn SystemSimulator>) {
    //     let clients: Vec<(ClientId, Client)> = self.clients.drain().collect();
    //     let mut handles = vec![];
    //     let mut adc: dyn ExtendedReservationProcessor = self.vrm.adcs.get_mut(&self.vrm.adc_master).expect("Did not find master ADC.");

    //     for (id, client) in clients {
    //         println!("Starting client: {}", id);

    //         let handle = tokio::spawn(async move {
    //             client.run(reservation_store, &mut adc).await;
    //         });

    //         handles.push(handle);
    //     }

    //     for handle in handles {
    //         let _ = handle.await;
    //     }

    //     println!("All clients have completed their reservations.");
    // }
}
