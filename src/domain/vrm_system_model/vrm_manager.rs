use std::{collections::HashMap, sync::Arc};
use tokio::time::{Duration, sleep};

use crate::{
    api::vrm_system_model_dto::vrm_dto::VrmDto,
    domain::{
        simulator::simulator::SystemSimulator,
        vrm_system_model::{
            grid_resource_management_system::{
                aci::AcI,
                adc::ADC,
                scheduler::workflow_scheduler_type::WorkflowSchedulerType,
                vrm_component_order::VrmComponentOrder,
                vrm_component_registry::{registry_client::RegistryClient, vrm_component_proxy::VrmComponentProxy},
                vrm_component_trait::VrmComponent,
            },
            reservation::{
                reservation::ReservationProceeding,
                reservation_store::{self, ReservationId, ReservationStore},
            },
            utils::id::{AdcId, ComponentId},
        },
    },
};

pub struct VrmManager {
    pub adc_master: VrmComponentProxy,
    pub unprocessed_reservations: Vec<(ReservationId, i64)>,
    pub open_reservations: Vec<ReservationId>,

    pub reservation_store: ReservationStore,
    pub simulator: Arc<dyn SystemSimulator>,
}

impl VrmManager {
    fn new(
        adc_master: VrmComponentProxy,
        unprocessed_reservations: Vec<(ReservationId, i64)>,
        reservation_store: ReservationStore,
        simulator: Arc<dyn SystemSimulator>,
    ) -> Self {
        VrmManager { adc_master, unprocessed_reservations, open_reservations: Vec::new(), reservation_store, simulator }
    }

    pub fn init_vrm_system(
        dto: VrmDto,
        unprocessed_reservations: Vec<ReservationId>,
        simulator: Arc<dyn SystemSimulator>,
        registry: RegistryClient,
        reservation_store: ReservationStore,
    ) -> Self {
        let mut proxies: HashMap<ComponentId, VrmComponentProxy> = HashMap::new();

        // Setup AcI Proxies (spawn all in own thread)
        for aci_dto in dto.aci {
            let aci = AcI::try_from((aci_dto, simulator.clone(), reservation_store.clone())).expect("Failed to create AcI");
            let component_box: Box<dyn VrmComponent + Send> = Box::new(aci);

            let proxy: VrmComponentProxy = registry.spawn_component(component_box);
            proxies.insert(proxy.get_id(), proxy);
        }

        let mut pending_adcs = dto.adc;
        let mut progress_made = true;
        let adc_master_id = ComponentId::new(dto.adc_master_id);
        let mut adc_master: Option<VrmComponentProxy> = None;

        // Setup ADC Proxies start bottom up (first init all children)(spawn all ADCs in there own thread)
        while !pending_adcs.is_empty() && progress_made {
            progress_made = false;
            let mut next_pending = Vec::new();

            for adc_dto in pending_adcs {
                let adc_id_str = adc_dto.id.clone();
                let children_ids: Vec<String> = adc_dto.children.clone();

                let all_children_ready = children_ids.iter().all(|child_id| proxies.contains_key(&ComponentId::new(child_id.clone())));

                if all_children_ready {
                    let mut children_proxies: Vec<VrmComponentProxy> = Vec::new();
                    for child_id in children_ids {
                        let proxy = proxies.get(&ComponentId::new(child_id)).unwrap().clone();

                        children_proxies.push(proxy.clone());
                    }

                    let workflow_scheduler = WorkflowSchedulerType::get_instance(WorkflowSchedulerType::HEFTSync, reservation_store.clone());

                    let vrm_component_order = VrmComponentOrder::OrderStartFirst;

                    let adc = ADC::new(
                        AdcId::new(adc_id_str),
                        children_proxies,
                        registry.clone(),
                        reservation_store.clone(),
                        workflow_scheduler,
                        vrm_component_order,
                        adc_dto.timeout,
                        simulator.clone(),
                        adc_dto.num_of_slots,
                        adc_dto.slot_width,
                    );
                    let component_box: Box<dyn VrmComponent + Send> = Box::new(adc);

                    let adc_proxy = registry.spawn_component(component_box);
                    if adc_master_id.compare(&adc_proxy.get_id()) {
                        adc_master = Some(adc_proxy.clone());
                    }
                    proxies.insert(adc_proxy.get_id(), adc_proxy);

                    progress_made = true;
                } else {
                    // Not ready yet (children missing)
                    next_pending.push(adc_dto);
                }
            }
            pending_adcs = next_pending;
        }

        if !pending_adcs.is_empty() {
            panic!("Failed to create all ADCs! Possible circular dependency or missing child ID.");
        }

        log::info!("System successfully initialized with {} components.", proxies.len());

        match adc_master {
            Some(adc_master) => VrmManager::new(
                adc_master,
                reservation_store.get_sorted_res_ids_with_arrival_time(unprocessed_reservations),
                reservation_store,
                simulator,
            ),
            None => panic!("Failed to find adc master. Possible mismatch of adcMasterId and actual id of the configuration."),
        }
    }

    pub async fn run_vrm(&mut self) {
        while !self.unprocessed_reservations.is_empty() {
            let (reservation_id, res_arrival_time) = self.unprocessed_reservations.remove(0);

            let now = self.simulator.get_current_time_in_s();
            log::info!("Now: {now}");
            if res_arrival_time > now {
                let wait_seconds = res_arrival_time - now;
                if wait_seconds > 0 {
                    sleep(Duration::from_secs(wait_seconds as u64)).await;
                }
            }

            if !self.reservation_store.contains(reservation_id) {
                panic!("Reservation {:?} was not added to the ReservationStore.", self.reservation_store.get_name_for_key(reservation_id));
            }

            self.process_reservation(reservation_id).await;
        }

        log::info!("VrmManager: Finished processing all unprocessed reservations.");
    }

    async fn process_reservation(&mut self, process_res_id: ReservationId) {
        log::info!("Try to submit Reservation {:?} the the master Adc.", self.reservation_store.get_name_for_key(process_res_id));
        let probe_reservations = self.adc_master.probe(process_res_id, None);

        probe_reservations.dump_reservation();
        // Step 1: Probe
        if probe_reservations.is_empty() {
            log::info!(
                "No probe results for Reservation {:?}, try reservation nevertheless.",
                self.reservation_store.get_name_for_key(process_res_id)
            );
        }

        if self.reservation_store.is_reservation_proceeding(process_res_id, ReservationProceeding::Probe) {
            log::info!("Reservation {:?}, canceled by user after probe.", self.reservation_store.get_name_for_key(process_res_id));
            return;
        }

        // let reserve_reservation = self.adc_master.reserve(process_res_id, None)

        // Step 2: Reserve
    }
}
