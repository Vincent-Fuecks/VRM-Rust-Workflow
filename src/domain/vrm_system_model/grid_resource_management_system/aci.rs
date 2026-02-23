use crate::api::rms_config_dto::rms_dto::RmsSystemWrapper;
use crate::api::vrm_system_model_dto::aci_dto::AcIDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;
use crate::domain::vrm_system_model::reservation::probe_reservations::{ProbeReservationComparator, ProbeReservations};
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationState};
use crate::domain::vrm_system_model::reservation::reservation_store::{NotificationListener, ReservationId, ReservationStore};
use crate::domain::vrm_system_model::reservation::reservation_sync_gate::SyncRegistry;
use crate::domain::vrm_system_model::reservation::vrm_state_listener::{self, VrmStateListener};
use crate::domain::vrm_system_model::rms::advance_reservation_trait::AdvanceReservationRms;
use crate::domain::vrm_system_model::rms::rms::RmsLoadMetric;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, ClientId, ComponentId, ReservationName, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;
use crate::domain::vrm_system_model::utils::statistics::ANALYTICS_TARGET;
use crate::error::ConversionError;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::i64;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum ScheduleID {
    FreeListSchedule,
    SlottedSchedule,
    SlottedScheduleResubmitFrag,
    SlottedSchedule12,
    SlottedSchedule12000,
    UnlimitedSchedule,
}

#[derive(Debug, Clone)]
pub struct ReservationContainer {
    /// Id of client, how submitted the request
    owner: ClientId,
    ///  Until which time the reservation has to be committed, if only reserved. VRM time in s.
    commit_deadline: i64,
    /// until which time the reservation has to finish execution. VRM time in s.
    execution_deadline: i64,
}

impl ReservationContainer {
    pub fn new(owner: ClientId, commit_deadline: i64, execution_deadline: i64) -> Self {
        Self { owner, commit_deadline, execution_deadline }
    }
}

#[derive(Debug, Default)]
struct ShadowScheduleReservations {
    inner_map: BTreeMap<ShadowScheduleId, HashMap<ReservationId, ReservationContainer>>,
}

impl ShadowScheduleReservations {
    pub fn new() -> Self {
        Self { inner_map: BTreeMap::new() }
    }

    pub fn get(&self, shadow_schedule_id: &ShadowScheduleId) -> Option<&HashMap<ReservationId, ReservationContainer>> {
        self.inner_map.get(shadow_schedule_id)
    }

    pub fn get_mut(&mut self, shadow_schedule_id: &ShadowScheduleId) -> Option<&mut HashMap<ReservationId, ReservationContainer>> {
        self.inner_map.get_mut(shadow_schedule_id)
    }

    pub fn contains_shadow_schedule_id(&self, shadow_schedule_id: &ShadowScheduleId) -> bool {
        self.inner_map.contains_key(shadow_schedule_id)
    }

    pub fn insert(
        &mut self,
        shadow_schedule_id: ShadowScheduleId,
        committed_reservations: &HashMap<ReservationId, ReservationContainer>,
        aci_id: &AciId,
    ) -> bool {
        if self.inner_map.contains_key(&shadow_schedule_id) {
            log::error!("AcI {}: ShadowScheduleId {} already exists. Delete it first.", aci_id, shadow_schedule_id);
            return false;
        }

        self.inner_map.insert(shadow_schedule_id, committed_reservations.clone());
        true
    }

    pub fn delete_shadow_schedule(&mut self, shadow_schedule_id: &ShadowScheduleId, aci_id: &AciId) -> bool {
        if self.inner_map.remove(shadow_schedule_id).is_none() {
            log::debug!("AcI {}: Could not delete ShadowScheduleId: {}. It did not exist.", aci_id, shadow_schedule_id);
            return false;
        }
        true
    }

    pub fn delete_reservation_container(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: &ShadowScheduleId,
    ) -> Option<ReservationContainer> {
        if let Some(map) = self.inner_map.get_mut(shadow_schedule_id) {
            return map.remove(&reservation_id);
        }
        None
    }
}

#[derive(Debug)]
pub struct AcI {
    pub id: AciId,
    adc_id: AdcId,
    commit_timeout: i64,
    rms_system: Box<dyn AdvanceReservationRms + Send>,
    shadow_schedule_reservations: ShadowScheduleReservations,
    committed_reservations: HashMap<ReservationId, ReservationContainer>,
    not_committed_reservations: HashMap<ReservationId, ReservationContainer>,
    open_probe_reservations: HashMap<ReservationId, Option<ShadowScheduleId>>,
    vrm_state_listener: VrmStateListener,
    sync_registry: SyncRegistry,

    simulator: Arc<dyn SystemSimulator>,
    reservation_store: ReservationStore,
}

impl TryFrom<(AcIDto, Arc<dyn SystemSimulator>, ReservationStore)> for AcI {
    type Error = ConversionError;

    fn try_from(args: (AcIDto, Arc<dyn SystemSimulator>, ReservationStore)) -> Result<Self, ConversionError> {
        let (dto, simulator, reservation_store) = args;

        let aci_id = AciId::new(dto.id.clone());
        let adc_id: AdcId = AdcId::new(dto.adc_id);

        let rms_system = RmsSystemWrapper::get_instance(dto.rms_system, simulator.clone(), aci_id.clone(), reservation_store.clone())?;

        let vrm_state_listener = VrmStateListener::new_empty();

        Ok(AcI {
            id: aci_id,
            adc_id: adc_id,
            commit_timeout: dto.commit_timeout,
            rms_system,
            shadow_schedule_reservations: ShadowScheduleReservations::new(),
            not_committed_reservations: HashMap::new(),
            committed_reservations: HashMap::new(),
            vrm_state_listener: vrm_state_listener,
            open_probe_reservations: HashMap::new(),
            sync_registry: SyncRegistry::new(),
            simulator: simulator.clone_box().into(),
            reservation_store: reservation_store.clone(),
        })

        // TODO
        // start background worker thread
        // Simulator.start(this);
    }
}

impl VrmComponent for AcI {
    fn get_id(&self) -> ComponentId {
        ComponentId::new(self.id.to_string())
    }

    fn get_total_capacity(&self) -> i64 {
        self.rms_system.get_total_capacity()
    }

    fn get_total_link_capacity(&self) -> i64 {
        self.rms_system.get_total_link_capacity()
    }

    fn get_total_node_capacity(&self) -> i64 {
        self.rms_system.get_total_node_capacity()
    }

    fn get_link_resource_count(&self) -> usize {
        self.rms_system.get_link_resource_count()
    }

    fn can_handel(&self, res: Reservation) -> bool {
        self.rms_system.can_handle_adc_request(res)
    }

    fn commit(&mut self, reservation_id: ReservationId) -> bool {
        log::debug!("AcI {}: is committing reservation {:?}", self.id, reservation_id);

        let arrival_time: i64 = self.simulator.get_current_time_in_ms();

        // Try to find reservation in not_committed
        let (container, id_to_commit) = match self.not_committed_reservations.remove(&reservation_id) {
            Some(reservation_container) => (reservation_container, reservation_id),
            None => {
                log::info!("No prior reserve for commit of {:?}. Attempting instant allocation.", reservation_id);

                // Check if RMS can handle it
                if !self.rms_system.can_handle_aci_request(self.reservation_store.clone(), reservation_id) {
                    self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                    self.log_stat("Commit".to_string(), reservation_id, arrival_time);
                    log::debug!(
                        "There was no reservation ({:?}) before the commit and the AcI ({}) can't handle the request.",
                        reservation_id,
                        self.id
                    );
                    return false;
                } else {
                    // Implicit Reserve
                    let possible_reservation_id = self.rms_system.reserve(reservation_id, None).unwrap_or(reservation_id);

                    if !self.reservation_store.is_reservation_state_at_least(possible_reservation_id, ReservationState::ReserveAnswer) {
                        log::debug!(
                            "Commit Reservation failed, because no former allocation was found. In AcI: {}, for reservation id {:?}.",
                            self.id,
                            possible_reservation_id
                        );
                        self.log_stat("Commit".to_string(), possible_reservation_id, arrival_time);
                        return false;
                    }

                    // Success: Create container and return new ID
                    let new_container = ReservationContainer {
                        owner: self.reservation_store.get_client_id(possible_reservation_id),
                        commit_deadline: i64::MAX,
                        execution_deadline: self.reservation_store.get_assigned_end(possible_reservation_id),
                    };

                    (new_container, possible_reservation_id)
                }
            }
        };

        self.rms_system.commit(id_to_commit);

        if self.reservation_store.get_state(id_to_commit) == ReservationState::Committed {
            self.committed_reservations.insert(id_to_commit, container);
            // TODO add event to clean up finished job
            // TODO from Java
            // Task, where commit_deadline or execution_deadline are reached
        }

        log::debug!("Committed reservation {:?} in AcI {}.", reservation_id, self.id);
        self.log_stat("Commit".to_string(), id_to_commit, arrival_time);
        return true;
    }

    fn commit_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        // TODO Add ReservationStore Listener
        let shadow_schedule_committed_reservations =
            self.shadow_schedule_reservations.get_mut(&shadow_schedule_id).expect("Committed Reservations where not found.").clone();

        let is_committed = self.rms_system.commit_shadow_schedule(&shadow_schedule_id);

        if is_committed {
            self.committed_reservations = shadow_schedule_committed_reservations;

            return self.shadow_schedule_reservations.delete_shadow_schedule(&shadow_schedule_id, &self.id);
        } else {
            panic!(
                "During the process of promoting a shadow schedule ({}) to the new master schedule in Aci: {} happened an error. The current shadow schedule of aci and the underlying rms are now not synchronized anymore.",
                shadow_schedule_id, self.id
            );
        }
    }

    fn create_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if self.rms_system.create_shadow_schedule(&shadow_schedule_id) {
            let aci_id = self.id.clone();
            if self.shadow_schedule_reservations.insert(shadow_schedule_id, &self.committed_reservations, &aci_id) {
                return true;
            } else {
                panic!(
                    "During the process of creating a new shadow schedule in Aci: {} happened an error. The current shadow schedule of aci and the underlying rms are now not synchronized.",
                    self.id
                )
            }
        }

        log::debug!(
            "The process of creating a new shadow schedule failed. However, the shadow schedule of aci: {} and the underlying rms are sill synchronized.",
            self.id
        );
        return false;
    }

    fn delete(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ReservationId {
        let arrival_time = self.simulator.get_current_time_in_ms();
        let container;

        if !shadow_schedule_id.is_none() {
            container = self.shadow_schedule_reservations.delete_reservation_container(reservation_id, &shadow_schedule_id.clone().unwrap());
        } else {
            container = self.not_committed_reservations.remove(&reservation_id);
        }

        if container.is_none() {
            log::info!("There was no reserve before the deletion of the reservation ({:?}) was performed.", reservation_id);
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
            if shadow_schedule_id.is_none() {
                self.log_stat("Delete".to_string(), reservation_id, arrival_time);
            }

            return reservation_id;
        }

        // Remove Task from RMS
        self.rms_system.delete_task(reservation_id, shadow_schedule_id.clone());

        if self.reservation_store.get_state(reservation_id) == ReservationState::Deleted {
            if shadow_schedule_id.is_none() {
                self.log_stat("Delete".to_string(), reservation_id, arrival_time);
            }
            return reservation_id;
        }

        // No Success
        self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
        if shadow_schedule_id.is_none() {
            self.log_stat("Delete".to_string(), reservation_id, arrival_time);
        }
        return reservation_id;
    }

    fn delete_shadow_schedule(&mut self, shadow_schedule_id: ShadowScheduleId) -> bool {
        if self.rms_system.delete_shadow_schedule(&shadow_schedule_id) {
            let aci_id = self.id.clone();
            if self.shadow_schedule_reservations.delete_shadow_schedule(&shadow_schedule_id, &aci_id) {
                return true;
            } else {
                panic!(
                    "During the process of deleting a new shadow schedule in Aci: {} happened an error. The current shadow schedule of aci and the underlying rms are now not synchronized.",
                    self.id
                );
            }
        }

        log::debug!(
            "The process of deleting a shadow schedule failed. However, the shadow schedule of aci: {} and the underlying rms are sill synchronized.",
            self.id
        );
        return false;
    }

    fn get_load_metric_up_to_date(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        self.rms_system.get_load_metric_up_to_date(start, end, shadow_schedule_id)
    }

    fn get_load_metric(&self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        self.rms_system.get_load_metric(start, end, shadow_schedule_id)
    }

    fn get_satisfaction(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        self.rms_system.get_fragmentation(start, end, shadow_schedule_id)
    }

    fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> RmsLoadMetric {
        self.rms_system.get_simulation_load_metric(shadow_schedule_id)
    }

    fn get_system_satisfaction(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        self.rms_system.get_system_fragmentation(shadow_schedule_id)
    }

    fn probe(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ProbeReservations {
        let arrival_time = self.simulator.get_current_time_in_ms();

        // Can Rms handle request in general?
        if !self.rms_system.can_handle_aci_request(self.reservation_store.clone(), reservation_id) {
            if shadow_schedule_id.is_none() {
                self.log_state_probe(-1, arrival_time);
            }
            return ProbeReservations::new(reservation_id, self.reservation_store.clone());
        }

        let prob_request_answer = self.rms_system.probe(reservation_id, shadow_schedule_id.clone());

        // Init ProbeReservation tracking -> Informs AcI if VrmComponent likes to reserve a ProbeReservation
        self.open_probe_reservations.insert(reservation_id, shadow_schedule_id.clone());
        self.vrm_state_listener.add(reservation_id);

        if prob_request_answer.is_empty() {
            if shadow_schedule_id.is_none() {
                self.log_state_probe(0, arrival_time);
            }

            return prob_request_answer;
        }

        if shadow_schedule_id.is_none() {
            self.log_state_probe(prob_request_answer.len() as i64, arrival_time);
        }

        return prob_request_answer;
    }

    fn probe_best(
        &mut self,
        reservation_id: ReservationId,
        shadow_schedule_id: Option<ShadowScheduleId>,
        probe_reservation_comparator: ProbeReservationComparator,
    ) -> ProbeReservations {
        log::debug!("In AcI {} a probeBest request based on reservation {:?} is requested.", self.id, reservation_id);

        let arrival_time = self.simulator.get_current_time_in_ms();

        if !self.rms_system.can_handle_aci_request(self.reservation_store.clone(), reservation_id) {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);

            if shadow_schedule_id.is_none() {
                self.log_stat("BestProbe".to_string(), reservation_id, arrival_time);
            }
            return ProbeReservations::new(reservation_id, self.reservation_store.clone());
        }

        let probe_best_answer = self.rms_system.probe_best(reservation_id, probe_reservation_comparator, shadow_schedule_id.clone());

        // Init ProbeReservation tracking -> Informs AcI if VrmComponent likes to reserve a ProbeReservation
        self.open_probe_reservations.insert(reservation_id, shadow_schedule_id);
        self.vrm_state_listener.add(reservation_id);

        return probe_best_answer;
    }

    fn reserve(&mut self, reservation_id: ReservationId, shadow_schedule_id: Option<ShadowScheduleId>) -> ReservationId {
        log::debug!("In AcI {} reserve reservation {:?} for ShadowScheduleId {:?}", self.id, reservation_id, shadow_schedule_id);

        let arrival_time = self.simulator.get_current_time_in_ms();

        if !self.rms_system.can_handle_aci_request(self.reservation_store.clone(), reservation_id) {
            self.reservation_store.update_state(reservation_id, ReservationState::Rejected);

            if shadow_schedule_id.is_none() {
                self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
            }
            return reservation_id;
        }

        let reserve_answer = self.rms_system.reserve(reservation_id, shadow_schedule_id.clone());

        match reserve_answer {
            None => {
                self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                if shadow_schedule_id.is_none() {
                    self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
                }
                return reservation_id;
            }
            Some(reservation_id_of_answer) => {
                if !self.reservation_store.is_reservation_state_at_least(reservation_id, ReservationState::ReserveAnswer) {
                    self.reservation_store.update_state(reservation_id_of_answer, ReservationState::Rejected);
                    if shadow_schedule_id.is_none() {
                        self.log_stat("Reserve".to_string(), reservation_id_of_answer, arrival_time);
                    }
                }

                let reservation_container = ReservationContainer::new(
                    self.reservation_store.get_client_id(reservation_id_of_answer),
                    self.reservation_store.get_assigned_end(reservation_id_of_answer),
                    self.simulator.get_current_time_in_s() + self.commit_timeout,
                );

                if !shadow_schedule_id.is_none() {
                    let mut new_committed_reservations: HashMap<ReservationId, ReservationContainer> = HashMap::new();
                    new_committed_reservations.insert(reservation_id_of_answer, reservation_container.clone());

                    if !self.shadow_schedule_reservations.insert(shadow_schedule_id.clone().unwrap(), &new_committed_reservations, &self.id) {
                        self.reservation_store.update_state(reservation_id, ReservationState::Rejected);
                        return reservation_id;
                    }
                }

                self.not_committed_reservations.insert(reservation_id_of_answer, reservation_container);

                if shadow_schedule_id.is_none() {
                    self.log_stat("Reserve".to_string(), reservation_id, arrival_time);
                }

                return reservation_id;
            }
        }
    }
}

impl AcI {
    pub fn log_stat(&mut self, command: String, reservation_id: ReservationId, arrival_time_at_aci: i64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let processing_time = self.simulator.get_current_time_in_ms() - arrival_time_at_aci;

        if let Some(res_handle) = self.reservation_store.get(reservation_id) {
            let (start, end, res_name, capacity, workload, state, proceeding, num_tasks) = {
                let res = res_handle.read().unwrap();

                let start = res.get_base_reservation().get_assigned_start();
                let end = res.get_base_reservation().get_assigned_end();
                let name = res.get_base_reservation().get_name().clone();
                let cap = res.get_base_reservation().get_reserved_capacity();
                let workload = res.get_base_reservation().get_task_duration() * cap;
                let state = res.get_base_reservation().get_state();
                let proceeding = res.get_base_reservation().get_reservation_proceeding();

                let mut tasks = 1;
                if res.is_workflow() {
                    tasks = res.as_workflow().unwrap().get_all_reservation_ids().len()
                }

                (start, end, name, cap, workload, state, proceeding, tasks)
            };

            let rms_load_metric = self.rms_system.get_load_metric_up_to_date(start, end, None);

            let node_utilization = rms_load_metric.node_load_metric.as_ref().map(|n| Some(n.utilization)).unwrap_or(None);

            let node_possible_capacity = rms_load_metric.node_load_metric.as_ref().map(|n| Some(n.possible_capacity)).unwrap_or(None);

            let network_utilization = rms_load_metric.link_load_metric.as_ref().map(|n| Some(n.utilization)).unwrap_or(None);

            let network_possible_capacity = rms_load_metric.link_load_metric.as_ref().map(|n| Some(n.possible_capacity)).unwrap_or(None);

            tracing::info!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished NodeLoadMetric",
                ComponentType = %self.id,
                NodeComponentUtilization = node_utilization,
                NodeComponentCapacity = node_possible_capacity,
                NetworkComponentUtilization = network_utilization,
                NetworkComponentCapacity = network_possible_capacity,
                ComponentFragmentation = self.rms_system.get_system_fragmentation(None),
                ReservationName = %res_name,
                ReservationCapacity = capacity,
                ReservationWorkload = workload,
                ReservationState = ?state,
                ReservationProceeding = ?proceeding,
                NumberOfTasks = num_tasks,
                Command = command,
                ProcessingTime = processing_time,
            );
        } else {
            // Handling in case reservation is missing (e.g. deleted/cleaned up)

            tracing::warn!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished (Reservation Missing/Deleted)",
                ComponentType = %self.id,
                ReservationId = ?reservation_id,
                Command = command,
                ProcessingTime = processing_time,
            );
        }
    }

    pub fn log_state_probe(&mut self, num_of_answers: i64, arrival_time_at_aci: i64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let processing_time = self.simulator.get_current_time_in_ms() - arrival_time_at_aci;

        tracing::info!(
            target: ANALYTICS_TARGET,
            Time = now,
            Command = "Commit".to_string(),
            ProbeAnswers = num_of_answers,
            ProcessingTime = processing_time,
        );
    }
}

impl NotificationListener for AcI {
    fn on_reservation_change(
        &mut self,
        reservation_id: ReservationId,
        res_name: ReservationName,
        old_state: ReservationState,
        new_state: ReservationState,
    ) {
        if old_state == ReservationState::ProbeReservation {
            let shadow_schedule_id = self.open_probe_reservations.remove(&reservation_id);

            if new_state == ReservationState::ReserveProbeReservation {
                log::debug!("AcIReserveProbeReservation: AcI {} performs Reserve of Reservation {:?}", self.id, res_name);

                self.reserve(reservation_id, shadow_schedule_id.unwrap());

                // Notify waiting ADC, that reserve operation was performed
                if let Some(gate) = self.sync_registry.get_gate(reservation_id) {
                    let final_state = self.reservation_store.get_state(reservation_id);
                    gate.notify(final_state, self.adc_id.clone().cast());
                }
            }
        }
    }
}
