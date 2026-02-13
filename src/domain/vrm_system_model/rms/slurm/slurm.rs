use reqwest::header::{HeaderMap, HeaderValue};
use std::{any::Any, sync::Arc};

use crate::{
    api::rms_config_dto::rms_dto::SlurmRmsDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            reservation::reservation_store::ReservationStore,
            rms::{
                rms::{Rms, RmsBase},
                slurm::{response::slurm_node::SlurmNodesResponse, slurm_endpoint::SlurmEndpoint},
            },
            utils::id::AciId,
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
    async fn new(
        dto: SlurmRmsDto,
        simulator: Arc<dyn SystemSimulator>,
        aci_id: AciId,
        reservation_store: ReservationStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();

        headers.insert("X-SLURM-USER-NAME", HeaderValue::from_str(&dto.user_name)?);
        headers.insert("X-SLURM-USER-TOKEN", HeaderValue::from_str(&dto.jwt_token)?);

        let client = reqwest::Client::builder().default_headers(headers).build()?;

        let response = client.get(format!("{}{:?}", dto.slurm_url, SlurmEndpoint::Nodes)).send().await?;
        let status = response.status();

        if status.is_success() {
            let slurm_nodes_response: SlurmNodesResponse = response.json().await?;

            for node in slurm_nodes_response.nodes {
                todo!()
            }
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
}

// impl TryFrom<(SlurmRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)> for SlurmRms {
//     type Error = Box<dyn std::error::Error>;

//     async fn try_from(args: (SlurmRmsDto, Arc<dyn SystemSimulator>, AciId, ReservationStore)) -> Result<Self, Self::Error> {
//         let (dto, simulator, aci_id, reservation_store) = args;

//         let mut headers = HeaderMap::new();
//         headers.insert("X-SLURM-USER-NAME", HeaderValue::from_str(&dto.user_name)?);
//         headers.insert("X-SLURM-USER-TOKEN", HeaderValue::from_str(&dto.jwt_token)?);

//         let client = reqwest::Client::builder()
//             .default_headers(headers)
//             .build()?;

//         let response = client.get(format!("{}{:?}", dto.slurm_url, SlurmEndpoint::Nodes)).send().await?;

//         let base = RmsBase { id: aci_id, schedule: (), shadow_schedules: (), slot_width: (), num_of_slots: (), resources: (), reservation_store }

//         Ok(SlurmRms::new(base, dto.slurm_url, dto.user_name, dto.jwt_token))
//     }
// }
