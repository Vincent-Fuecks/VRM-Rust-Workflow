use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};

use std::collections::HashMap;
use std::i64;
use std::{any::Any, str::FromStr, sync::Arc};

use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::rms_node_network_trait::Helper;
use crate::domain::vrm_system_model::schedule::schedule_trait::Schedule;
use crate::domain::vrm_system_model::schedule::slotted_schedule::strategy::link::topology::NetworkTopology;
use crate::domain::vrm_system_model::scheduler_type::ScheduleContext;
use crate::domain::vrm_system_model::utils::id::{ResourceName, ShadowScheduleId, SlottedScheduleId};
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
            scheduler_type::SchedulerType,
            utils::id::{AciId, RouterId},
        },
    },
};

#[derive(Debug)]
pub struct SlurmRms {
    pub base: RmsBase,
    pub node_schedule: Box<dyn Schedule>,
    pub network_schedule: Box<dyn Schedule>,
    pub node_shadow_schedule: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
    pub network_shadow_schedule: HashMap<ShadowScheduleId, Box<dyn Schedule>>,
    pub slurm_url: String,
    pub user_name: String,
    pub jwt_token: String,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct SlurmTopology {
    pub switch_name: RouterId,
    pub switches: Vec<RouterId>,
    pub nodes: Vec<ResourceName>,
    pub link_speed: i64,
}

impl Helper for SlurmRms {
    fn get_node_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &self.node_shadow_schedule
    }

    fn get_mut_network_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &mut self.network_shadow_schedule
    }

    fn get_network_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &self.node_shadow_schedule
    }

    fn get_mut_node_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Box<dyn Schedule>> {
        &mut self.node_shadow_schedule
    }

    fn get_node_schedule(&self) -> &Box<dyn Schedule> {
        &self.node_schedule
    }

    fn get_mut_node_schedule(&mut self) -> &mut Box<dyn Schedule> {
        &mut self.node_schedule
    }

    fn get_network_schedule(&self) -> &Box<dyn Schedule> {
        &self.network_schedule
    }

    fn get_mut_network_schedule(&mut self) -> &mut Box<dyn Schedule> {
        &mut self.network_schedule
    }

    fn set_node_schedule(&mut self, new_node_schedule: Box<dyn Schedule>) {
        self.node_schedule = new_node_schedule;
    }

    fn set_network_schedule(&mut self, new_network_schedule: Box<dyn Schedule>) {
        self.network_schedule = new_network_schedule;
    }
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

    fn get_mut_active_schedule(&mut self, shadow_schedule_id: Option<ShadowScheduleId>, reservation_id: ReservationId) -> &mut Box<dyn Schedule> {
        if self.base.reservation_store.is_link(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.network_shadow_schedule.get_mut(&id).expect("network_shadow_schedule contains ShadowSchedule."),
                None => &mut self.network_schedule,
            }
        } else if self.base.reservation_store.is_node(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.node_shadow_schedule.get_mut(&id).expect("node_shadow_schedule contains ShadowSchedule."),
                None => &mut self.node_schedule,
            }
        } else {
            panic!(
                "RmsSimulatorErrorNoScheduleForReservation: The rms RmsSimulator has no Scheduler for Reservation type {:?}. ReservationName: {:?} ShadowScheduleId {:?}",
                self.base.reservation_store.get_type(reservation_id),
                self.base.reservation_store.get_name_for_key(reservation_id),
                shadow_schedule_id
            );
        }
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

            // Setup Node Schedule
            let mut schedule_capacity = 0;

            // Add nodes to ResourceStore
            for node in nodes.iter() {
                schedule_capacity += node.cpus;
                resource_store.add_node(NodeResource::new(node.name.clone(), node.cpus));
            }

            let name = format!("AcI: {}, RmsType: {}, RmsName: {}", aci_id, "Slurm".to_string(), dto.id);
            let schedule_context = ScheduleContext {
                id: SlottedScheduleId::new(name.clone()),
                number_of_slots: dto.num_of_slots,
                slot_width: dto.slot_width,
                capacity: schedule_capacity,
                simulator: simulator.clone(),
                reservation_store: reservation_store.clone(),
            };

            let scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
            let node_schedule = scheduler_type.get_instance(schedule_context);

            // Setup Network Schedule
            // Adds Links to Resource Store
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

            let schedule_context = ScheduleContext {
                id: SlottedScheduleId::new(name.clone()),
                number_of_slots: dto.num_of_slots,
                slot_width: dto.slot_width,
                capacity: i64::MAX,
                simulator: simulator.clone(),
                reservation_store: reservation_store.clone(),
            };

            let mut scheduler_type = SchedulerType::from_str(&dto.scheduler_typ)?;
            scheduler_type = scheduler_type.get_network_scheduler_variant(topology, resource_store.clone());
            let network_schedule = scheduler_type.get_instance(schedule_context);

            let base = RmsBase::new(aci_id, "Slurm".to_string(), reservation_store, resource_store.clone());

            Ok(SlurmRms {
                base: base,
                node_schedule,
                network_schedule,
                node_shadow_schedule: HashMap::new(),
                network_shadow_schedule: HashMap::new(),
                jwt_token: dto.jwt_token,
                slurm_url: dto.slurm_url,
                user_name: dto.user_name,
                client,
            })
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
