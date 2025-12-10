use crate::api::workflow_dto::client_dto::SystemModelDto;
use crate::domain::vrm_system_model::workflow::client::SystemModel;
use crate::error::Result;
use crate::loader::parser::parse_json_file;

pub mod api;
pub mod domain;
pub mod error;
pub mod loader;
pub mod logger;

pub fn generate_system_model(file_path: &str) -> Result<SystemModel> {
    logger::init();
    log::info!("Logger initialized. Starting SystemModel construction.");

    let root_dto: SystemModelDto = parse_json_file::<SystemModelDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let system_model = SystemModel::from_dto(root_dto)?;
    log::info!("Internal SystemModel constructed successfully.");

    Ok(system_model)
}

fn main() {
    let file_path: &str = "/home/vincent/Desktop/Repository/VRM-Rust-Workflow/src/data/test/test_workflow_loading_01.json";

    let _model: Result<SystemModel> = generate_system_model(file_path);
}

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
