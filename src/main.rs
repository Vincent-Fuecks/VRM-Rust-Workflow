use std::sync::Arc;

use vrm_rust_workflow::domain::vrm_system_model::reservation::reservation_store::ReservationStore;

use crate::domain::vrm_system_model::system::System;

use crate::api::vrm_system_model_dto::vrm_dto::VrmDto;
use crate::api::workflow_dto::client_dto::ClientsDto;
use crate::domain::simulator::simulator::Simulator;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::client::client::Clients;
use crate::domain::vrm_system_model::utils::statistics::init_tracing;
use crate::domain::vrm_system_model::vrm_system_model::Vrm;
use crate::error::Result;
use crate::loader::parser::parse_json_file;

pub mod api;
pub mod domain;
pub mod error;
pub mod loader;
pub mod logger;

pub fn get_clients(file_path: &str, simulator: Arc<dyn SystemSimulator>) -> Result<Clients> {
    log::info!("Starting ClientsDto construction.");

    let root_dto: ClientsDto = parse_json_file::<ClientsDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let system_model = Clients::from_dto(root_dto, simulator)?;
    log::info!("Internal SystemModel was constructed successfully.");

    Ok(system_model)
}

pub fn get_vrm(file_path: &str, simulator: Arc<dyn SystemSimulator>) -> Result<Vrm> {
    log::info!("Starting VrmDto construction.");

    let root_dto: VrmDto = parse_json_file::<VrmDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let system_model = Vrm::from_dto(root_dto, simulator.clone_box().0)?;
    log::info!("Internal Vrm was constructed successfully.");

    Ok(system_model)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init Logging
    logger::init();

    // Init statistics logging
    let _guard = init_tracing("statistics", "system_statistics");
    let simulator = Arc::new(Simulator::new(true));
    let reservation_store = ReservationStore::new(None);

    let file_path_workflows: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_loading_01.json";
    let file_path_vrm: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/vrm.json";

    let clients = get_clients(file_path_workflows, simulator.clone()).expect("TODO");
    let vrm = get_vrm(file_path_vrm, simulator.clone_box().into()).expect("TODO");

    let mut system = System::new(clients.clients, vrm);

    // This will start all clients and wait for them to finish
    system.run_all_clients(reservation_store, simulator).await;

    Ok(())
}

// fn main() {
//     let file_path: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_loading_01.json";

//     let _model: Result<SystemModel> = generate_system_model(file_path);

//     // init statistics logging
//     let _guard = init_tracing("statistics", "system_statistics");
// }

// log::info!("Logger initialized. Starting SystemModel construction.");

// // This path comes from your original main.rs
// let file_path = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/workflows.json";
// let json_str = parse_json_file(file_path);

// match json_str {
//     Some(json_str) => {
//         log::info!("Loading from path: '{}'...", file_path);

//         // 1. Deserialize the JSON string into the DTOs
//         let root_dto: RootDto = match serde_json::from_str(&json_str) {
//             Ok(dto) => dto,
//             Err(e) => {
//                 log::error!("Failed to parse workflow_dto.rs: {}", e);
//                 return;
//             }
//         };

//         log::debug!("JSON DTOs parsed successfully. Building internal model.");

//         // 2. Construct the internal graph model from the DTOs
//         match SystemModel::from_dto(root_dto) {
//             Ok(system_model) => {
//                 log::info!("Internal SystemModel constructed successfully.");

//                 // 3. Print the resulting model (uses the Debug trait)
//                 // This will be a very large output, so we use debug!
//                 // Change to info! or println! if you want to see it.
//                 log::debug!("{:#?}", system_model);

//                 // Example of how to access the model:
//                 if let Some(client) = system_model.clients.values().next() {
//                     log::info!("Loaded Client ID: {}", client.id);
//                     if let Some(workflow) = client.workflows.values().next() {
//                         log::info!("Workflow '{}' has {} nodes and {} data dependencies.",
//                             workflow.base.job_name,
//                             workflow.nodes.len(),
//                             workflow.data_dependencies.len()
//                         );
//                     }
//                 }
//             }
//             Err(e) => {
//                 log::error!("Failed to build internal model: {}", e);
//             }
//         }
//     }
//     None => {
//         log::error!("Error during loading of json file!");
//     }
// }
// }
