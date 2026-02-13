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
                advance_reservation_trait::AdvanceReservationRms,
                rms::{Rms, RmsBase},
                slurm::{response::slurm_node::SlurmNodesResponse, slurm_endpoint::SlurmEndpoint},
            },
            schedule::slotted_schedule::network_slotted_schedule::topology::{NetworkTopology, TopologyContext},
            scheduler_type::SchedulerType,
            utils::id::{AciId, RouterId},
        },
    },
};

#[derive(Debug)]
pub struct SlurmRms {
    pub base: RmsBase,
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
    pub async fn new(
        dto: SlurmRmsDto,
        simulator: Arc<dyn SystemSimulator>,
        aci_id: AciId,
        reservation_store: ReservationStore,
    ) -> Result<Box<dyn AdvanceReservationRms>, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("X-SLURM-USER-NAME", HeaderValue::from_str(&dto.user_name)?);
        headers.insert("X-SLURM-USER-TOKEN", HeaderValue::from_str(&dto.jwt_token)?);

        let client = reqwest::Client::builder().default_headers(headers).build()?;
        let response = client.get(format!("{}{:?}", dto.slurm_url, SlurmEndpoint::Nodes)).send().await?;
        let status = response.status();

        if status.is_success() {
            let nodes_endpoin_response: SlurmNodesResponse = response.json().await?;

            let (nodes, links) = SlurmRms::get_nodes_and_links(&dto, &nodes_endpoin_response);

            let topology_context = TopologyContext::new(links, nodes, dto.slot_width, dto.num_of_slots);

            let topology = NetworkTopology::new(topology_context, simulator, aci_id, reservation_store);

            let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
            scheduler_type = scheduler_type.get_network_scheduler_variant(topology);

            let base = RmsBase::






        } else {
            let body_text = response.text().await?;
            log::error!(
                "Initialisation of Rms by AcI {} of Rms {} failed. Becasue the returned rms response was not successfull. The following request was unsuccessfull:\nX-SLURM-USER-NAME: <<{}>>\nSlurm-URL: <<{}>>\nSlurm-Requesed-Endpoint: <<{:?}>>\nResponse-Status-Code: <<{}>>\nResponse-Body: <<{}>>\n\nPlease also consider, that your provided jwt-token is still valied.",
                aci_id,
                dto.id,
                dto.user_name,
                dto.slurm_url,
                SlurmEndpoint::Nodes,
                status,
                body_text
            );
        }
        todo!();
    }

    async fn get_cluster_topology() -> HashMap<RouterId, HashSet<RouterId>> {
        todo!()
    }
}
