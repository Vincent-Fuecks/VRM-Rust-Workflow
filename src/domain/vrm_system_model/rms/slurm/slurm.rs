use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

use std::i64;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use crate::domain::vrm_system_model::reservation::reservation::ReservationState;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::utils::id::ResourceName;
use crate::{
    api::rms_config_dto::rms_dto::SlurmRmsDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            reservation::reservation_store::ReservationStore,
            rms::{
                rms::{Rms, RmsBase, RmsContext},
                slurm::{response::slurm_node::SlurmNodesResponse, slurm_endpoint::SlurmEndpoint},
            },
            schedule::slotted_schedule::network_slotted_schedule::topology::NetworkTopology,
            scheduler_type::SchedulerType,
            utils::id::{AciId, RouterId},
        },
    },
};

#[derive(Debug)]
pub struct SlurmRms {
    pub base: RmsBase,
    client: Client,
    pub slurm_url: String,
    pub user_name: String,
    pub jwt_token: String,
}

#[derive(Debug, Clone)]
pub struct SlurmTopology {
    pub switch_name: RouterId,
    pub switches: Vec<RouterId>,
    pub nodes: Vec<ResourceName>,
    pub link_speed: i64,
}

impl Rms for SlurmRms {
    fn get_base(&self) -> &RmsBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut RmsBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SlurmRms {
    pub fn new(
        dto: SlurmRmsDto,
        simulator: Arc<dyn SystemSimulator>,
        aci_id: AciId,
        reservation_store: ReservationStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("X-SLURM-USER-NAME", HeaderValue::from_str(&dto.user_name)?);
        headers.insert("X-SLURM-USER-TOKEN", HeaderValue::from_str(&dto.jwt_token)?);

        let client = Client::builder().default_headers(headers).build()?;

        let response = client.get("http://localhost:6820/slurm/v0.0.41/nodes").send()?;
        let status = response.status();

        if status.is_success() {
            let nodes_response: SlurmNodesResponse = response.json()?;

            let (nodes, links) = SlurmRms::get_nodes_and_links(&dto, &nodes_response);
            let resource_store = ResourceStore::new();

            let topology = NetworkTopology::new(
                &links,
                &nodes,
                dto.slot_width,
                dto.num_of_slots,
                simulator.clone(),
                aci_id.clone(),
                reservation_store.clone(),
                resource_store.clone(),
            );

            let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
            scheduler_type = scheduler_type.get_network_scheduler_variant(topology, resource_store.clone());

            let rms_context = RmsContext {
                aci_id: aci_id,
                rms_type: "SLURM".to_string(),
                schedule_capacity: i64::MAX,
                slot_width: dto.slot_width,
                num_of_slots: dto.num_of_slots,
                nodes: nodes,
                reservation_store,
                simulator,
                schedule_type: scheduler_type,
            };

            let base = RmsBase::new(rms_context, resource_store);

            Ok(SlurmRms { base: base, client, jwt_token: dto.jwt_token, slurm_url: dto.slurm_url, user_name: dto.user_name })
        } else {
            let body_text = response.text();
            panic!(
                "Initialisation of Rms by AcI {} of Rms {} failed. Becasue the returned rms response was not successfull. The following request was unsuccessfull:\nX-SLURM-USER-NAME: <<{}>>\nSlurm-URL: <<{}>>\nSlurm-Requesed-Endpoint: <<{:?}>>\nResponse-Status-Code: <<{}>>\nResponse-Body: <<{:?}>>\n\nPlease also consider, that your provided jwt-token is still valied.",
                aci_id,
                dto.id,
                dto.user_name,
                dto.slurm_url,
                SlurmEndpoint::Nodes,
                status,
                body_text
            );
        }
    }
}

impl AdvanceReservationRms for SlurmRms {
    fn can_rms_handle_reservation(&self, reservation_id: ReservationId) -> bool {
        if self.get_base().reservation_store.is_link(reservation_id) || self.get_base().reservation_store.is_node(reservation_id) {
            true
        } else {
            log::debug!(
                "The Reservation {:?} was submitted to the SlurmRms, which is either a NodeReservation or LinkReservation",
                self.get_base().reservation_store.get_name_for_key(reservation_id)
            );
            self.get_base().reservation_store.update_state(reservation_id, ReservationState::Rejected);
            false
        }
    }
}
