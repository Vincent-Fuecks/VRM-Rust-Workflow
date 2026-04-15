use bimap::{BiHashMap, BiMap};
use reqwest::header::{self, HeaderMap, HeaderValue};
use std::collections::{HashMap, HashSet};
use std::i64;
use std::sync::RwLock;
use std::{any::Any, str::FromStr, sync::Arc};
use tokio::time::{Duration, MissedTickBehavior, interval};

use crate::domain::vrm_system_model::reservation::node_reservation::NodeReservation;
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState, ReservationTrait};
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationId;
use crate::domain::vrm_system_model::resource::node_resource::NodeResource;
use crate::domain::vrm_system_model::resource::resource_store::ResourceStore;
use crate::domain::vrm_system_model::rms::rms_node_network_trait::Helper;
use crate::domain::vrm_system_model::rms::slurm::slurm_rest_client::SlurmRestApiClient;
use crate::domain::vrm_system_model::schedule::schedule_trait::Schedule;
use crate::domain::vrm_system_model::schedule::slotted_schedule::strategy::link::topology::NetworkTopology;
use crate::domain::vrm_system_model::scheduler_type::ScheduleContext;
use crate::domain::vrm_system_model::utils::config::SCHEDULE_SYNC_TIMEINTERVAL_S;
use crate::domain::vrm_system_model::utils::id::{ResourceName, RmsId, ShadowScheduleId, SlottedScheduleId};
use crate::{
    api::rms_config_dto::rms_dto::SlurmRmsDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            reservation::reservation_store::ReservationStore,
            rms::{
                rms::{Rms, RmsBase},
                slurm::slurm_endpoint::SlurmEndpoint,
            },
            scheduler_type::SchedulerType,
            utils::id::{AciId, RouterId},
        },
    },
};

use super::response::nodes::SlurmNodesResponse;
use super::response::tasks::SlurmTaskResponse;
use super::rms_trait::SlurmRestApi;

#[derive(Debug)]
pub struct SlurmRms {
    pub base: RmsBase,
    pub aci_id: AciId,
    pub slurm_rest_client: SlurmRestApiClient,

    // Master Schedules
    pub node_schedule: Arc<RwLock<Box<dyn Schedule>>>,
    pub network_schedule: Arc<RwLock<Box<dyn Schedule>>>,

    // Shadow schedules for simulations
    pub node_shadow_schedule: HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>>,
    pub network_shadow_schedule: HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>>,

    // Mapping between VRM ReservationId and Slurm Task Id
    pub task_mapping: Arc<RwLock<BiMap<ReservationId, u32>>>,
}

#[derive(Debug, Clone)]
pub struct SlurmTopology {
    pub switch_name: RouterId,
    pub switches: Vec<RouterId>,
    pub nodes: Vec<ResourceName>,
    pub link_speed: i64,
}

impl Helper for SlurmRms {
    fn get_node_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>> {
        &self.node_shadow_schedule
    }

    fn get_mut_network_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>> {
        &mut self.network_shadow_schedule
    }

    fn get_network_shadow_schedule(&self) -> &HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>> {
        &self.network_shadow_schedule
    }

    fn get_mut_node_shadow_schedule(&mut self) -> &mut HashMap<ShadowScheduleId, Arc<RwLock<Box<dyn Schedule>>>> {
        &mut self.node_shadow_schedule
    }

    fn get_node_schedule(&self) -> Arc<RwLock<Box<dyn Schedule>>> {
        self.node_schedule.clone()
    }

    fn get_network_schedule(&self) -> Arc<RwLock<Box<dyn Schedule>>> {
        self.network_schedule.clone()
    }

    fn set_node_schedule(&mut self, new_node_schedule: Arc<RwLock<Box<dyn Schedule>>>) {
        self.node_schedule = new_node_schedule;
    }

    fn set_network_schedule(&mut self, new_network_schedule: Arc<RwLock<Box<dyn Schedule>>>) {
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

    fn get_active_schedule(&self, shadow_schedule_id: Option<ShadowScheduleId>, reservation_id: ReservationId) -> Arc<RwLock<Box<dyn Schedule>>> {
        if self.base.reservation_store.is_link(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.network_shadow_schedule.get(&id).expect("network_shadow_schedule contains ShadowSchedule.").clone(),
                None => self.network_schedule.clone(),
            }
        } else if self.base.reservation_store.is_node(reservation_id) {
            match shadow_schedule_id {
                Some(id) => self.node_shadow_schedule.get(&id).expect("node_shadow_schedule contains ShadowSchedule.").clone(),
                None => self.node_schedule.clone(),
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
    pub async fn new(
        dto: SlurmRmsDto,
        simulator: Arc<dyn SystemSimulator>,
        aci_id: AciId,
        reservation_store: ReservationStore,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("X-SLURM-USER-NAME", HeaderValue::from_str(&dto.rest_api_config.user_name)?);
        headers.insert("X-SLURM-USER-TOKEN", HeaderValue::from_str(&dto.rest_api_config.jwt_token)?);
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder().default_headers(headers).build()?;

        let response = client.get("http://localhost:6820/slurm/v0.0.41/nodes").send().await?;
        let status = response.status();

        if status.is_success() {
            let nodes_response: SlurmNodesResponse = response.json().await?;

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
            let node_schedule = Arc::new(RwLock::new(scheduler_type.get_instance(schedule_context)));

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
            let network_schedule = Arc::new(RwLock::new(scheduler_type.get_instance(schedule_context)));

            let base = RmsBase::new(aci_id.clone(), "Slurm".to_string(), reservation_store, resource_store.clone());
            let rest_api_client = SlurmRestApiClient::new(dto.rest_api_config)?;

            Ok(SlurmRms {
                base: base,
                aci_id: aci_id,
                node_schedule,
                network_schedule,
                node_shadow_schedule: HashMap::new(),
                network_shadow_schedule: HashMap::new(),
                slurm_rest_client: rest_api_client,
                task_mapping: Arc::new(RwLock::new(BiMap::new())),
            })
        } else {
            let body_text = response.text().await?;
            panic!(
                "Initialization of Rms by AcI {} of Rms {} failed. Because the returned rms response was not successful. The following request was unsuccessful:\nX-SLURM-USER-NAME: <<{}>>\nSlurm-URL: <<{}>>\nSlurm-Requested-Endpoint: <<{:?}>>\nResponse-Status-Code: <<{}>>\nResponse-Body: <<{:?}>>\n\nPlease also consider, that your provided jwt-token is still valid.",
                aci_id,
                dto.id,
                dto.rest_api_config.user_name,
                dto.rest_api_config.base_url,
                SlurmEndpoint::Nodes,
                status,
                body_text
            );
        }
    }
}

impl SlurmRms {
    /// Starts the background synchronization loop.
    pub fn start_sync(&self) {
        let node_schedule = self.node_schedule.clone();
        let slurm_rest_client = self.slurm_rest_client.clone();
        let task_mapping = self.task_mapping.clone();
        let resource_store = self.base.resource_store.clone();
        let reservation_store = self.base.reservation_store.clone();
        let rms_id = self.base.id.clone();
        let aci_id = self.aci_id.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(SCHEDULE_SYNC_TIMEINTERVAL_S));
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::perform_sync(&slurm_rest_client, &resource_store, &reservation_store, &node_schedule, &task_mapping, &rms_id, &aci_id).await
                {
                    log::error!("Slurm Schedule Sync Error: {:?}", e);
                }
            }
        });
    }

    async fn perform_sync(
        client: &SlurmRestApiClient,
        resource_store: &ResourceStore,
        reservation_store: &ReservationStore,
        node_schedule: &Arc<RwLock<Box<dyn Schedule>>>,
        task_mapping: &Arc<RwLock<BiHashMap<ReservationId, u32>>>,
        rms_id: &RmsId,
        aci_id: &AciId,
    ) -> anyhow::Result<()> {
        let slurm_nodes = client.get_nodes().await?;
        let slurm_tasks = client.get_tasks().await?;

        let node_resources: Vec<NodeResource> =
            slurm_nodes.nodes.iter().map(|node| NodeResource::new(ResourceName::new(node.name.clone()), node.cpus as i64)).collect();

        // Update Node in ResourceStore
        resource_store.update_nodes(node_resources);
        Self::update_reservations(reservation_store, task_mapping, node_schedule, slurm_tasks, rms_id, aci_id);

        Ok(())
    }

    pub fn update_reservations(
        reservation_store: &ReservationStore,
        task_mapping: &Arc<RwLock<BiHashMap<ReservationId, u32>>>,
        node_schedule: &Arc<RwLock<Box<dyn Schedule>>>,
        slurm_tasks: SlurmTaskResponse,
        rms_id: &RmsId,
        aci_id: &AciId,
    ) {
        let active_slurm_ids: HashSet<u32> = slurm_tasks.jobs.iter().map(|job| job.job_id).collect();
        let mut external_reservations = Vec::new();

        // Tasks deleted by the RMS scheduling logic
        let to_remove: Vec<(ReservationId, u32)> = {
            let mapping = task_mapping.read().unwrap();
            mapping
                .iter()
                .filter(|(_, slurm_task_id)| !active_slurm_ids.contains(slurm_task_id))
                .map(|(reservation_id, slurm_task_id)| (reservation_id.clone(), *slurm_task_id))
                .collect()
        };

        // Process Task Updates
        for slurm_task in slurm_tasks.jobs {
            // Task is tracked in Schedule
            if let Some(reservation_id) = task_mapping.read().unwrap().get_by_right(&slurm_task.job_id) {
                if let Some(slurm_task_states) = slurm_task.job_state {
                    if slurm_task_states.is_empty() {
                        log::debug!(
                            "The slurm job {:?} running on RMS {:?} contains no valid state. Possible due to a Slurm cluster failure.",
                            slurm_task.job_id,
                            rms_id
                        );
                    } else if slurm_task_states.len() > 1 {
                        log::debug!(
                            "The slurm job {:?} running on RMS {:?} contains multiple job states {:?}, currently only the first state is taken into account.",
                            slurm_task.job_id,
                            rms_id,
                            slurm_task_states
                        );
                    } else {
                        if let Ok(slurm_task_reservation_state) = ReservationState::from_slurm_task_state(slurm_task_states.get(0).unwrap()) {
                            // Task state in RMS and Schedule are different
                            if reservation_store.get_state(reservation_id.clone()) != slurm_task_reservation_state {
                                reservation_store.update_state(*reservation_id, slurm_task_reservation_state);
                            }
                        }
                    }
                }
            } else {
                // Aggregate External Reservations
                let node_reservation = Reservation::Node(NodeReservation::from_slurm(&slurm_task, aci_id.clone().cast()));
                external_reservations.push((slurm_task.job_id, node_reservation));
            }
        }

        // Deletes Reservations by setting them into the Deleted State
        if !to_remove.is_empty() {
            let mut mapping = task_mapping.write().unwrap();
            for (res_id, slurm_task_id) in to_remove {
                reservation_store.update_state(res_id, ReservationState::Deleted);
                mapping.remove_by_right(&slurm_task_id);
            }
        }

        // Add External Task to Schedule
        Self::update_schedule(reservation_store, task_mapping, external_reservations);
    }

    pub fn update_schedule(
        reservation_store: &ReservationStore,
        task_mapping: &Arc<RwLock<BiHashMap<ReservationId, u32>>>,
        external_reservations: Vec<(u32, Reservation)>,
    ) {
        for (slurm_task_id, res) in external_reservations {
            log::debug!("INSERT EXTERNAL: RESERVATION {:?} into ReservationStore", res.get_name());

            let res_id = reservation_store.add(res);
            task_mapping.write().unwrap().insert(res_id, slurm_task_id);

            // TODO Insert and re-schedule other reservations in Schedule
        }

        todo!("Update of Schedule, due to external Reservations is currently not supported.")
    }
}
