use crate::domain::vrm_system_model::utils::statistics::AnalyticsSystem;
use crate::domain::vrm_system_model::vrm_manager::VrmManager;

use crate::domain::vrm_system_model::client::client::Clients;

use crate::api::workflow_dto::client_dto::ClientsDto;
use crate::domain::simulator::simulator::Simulator;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_registry::registry_client::RegistryClient;
use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;

use std::sync::Arc;

use crate::api::vrm_system_model_dto::vrm_dto::VrmDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::error::Result;
use crate::loader::parser::parse_json_file;

pub mod api;
pub mod domain;
pub mod error;
pub mod loader;
pub mod logger;

pub fn get_vrm_dto(file_path: &str) -> Result<VrmDto> {
    log::info!("Starting VrmDto construction.");

    let root_dto: VrmDto = parse_json_file::<VrmDto>(file_path)?;
    log::info!("JSON file parsed successfully.");
    Ok(root_dto)
}

#[tokio::main]
async fn main() {
    // Init Logging
    logger::init();
    let log_file_path = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/statistics/analytics.csv".to_string();
    AnalyticsSystem::init(log_file_path);

    let file_path_workflows: &str = "src/data/test/test_workflow_with_simple_co_allocation_graph.json";
    let file_path_vrm: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/vrm.json";
    let reservation_store = ReservationStore::new(None);

    let vrm_dto = get_vrm_dto(file_path_vrm).expect("Failed to load VRM DTO");
    let simulator_dto = vrm_dto.simulator.clone();
    let unprocessed_reservations = Clients::get_clients(file_path_workflows, reservation_store.clone()).expect("TODO").unprocessed_reservations;

    let registry = RegistryClient::new();
    let simulator: Arc<dyn SystemSimulator> = Arc::new(Simulator::new(simulator_dto));

    let mut vrm_manager = VrmManager::init_vrm_system(vrm_dto, unprocessed_reservations, simulator.clone(), registry, reservation_store.clone());
    vrm_manager.run_vrm().await;

    // Prevent main from exiting immediately so threads can run
    std::thread::park();
}
