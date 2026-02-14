use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use crate::{
    api::rms_config_dto::rms_dto::SlurmRmsDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            reservation::reservation_store::ReservationStore,
            resource::resource_trait::ResourceId,
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
    pub nodes: Vec<ResourceId>,
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

        let response = client.get(format!("{}{:?}", dto.slurm_url, SlurmEndpoint::Nodes)).send()?;
        let status = response.status();

        if status.is_success() {
            let nodes_response: SlurmNodesResponse = response.json()?;

            let (nodes, links) = SlurmRms::get_nodes_and_links(&dto, &nodes_response);

            let topology =
                NetworkTopology::new(&links, &nodes, dto.slot_width, dto.num_of_slots, simulator.clone(), aci_id.clone(), reservation_store.clone());

            let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
            scheduler_type = scheduler_type.get_network_scheduler_variant(topology);

            let rms_context = RmsContext {
                aci_id: aci_id,
                num_of_slots: dto.num_of_slots,
                reservation_store,
                rms_type: "SLURM".to_string(),
                schedule_type: scheduler_type,
                simulator,
                slot_width: dto.slot_width,
            };

            let base = RmsBase::new(rms_context, &nodes, &links);

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

    async fn get_cluster_topology() -> HashMap<RouterId, HashSet<RouterId>> {
        todo!()
    }
}
