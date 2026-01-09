use std::sync::Arc;

use crate::api::workflow_dto::client_dto::ClientsDto;
use crate::domain::simulator::simulator::SystemSimulator;
use crate::domain::vrm_system_model::client::client::Clients;
use crate::error::Result;
use crate::loader::parser::parse_json_file;

pub mod api;
pub mod domain;
pub mod error;
pub mod loader;
pub mod logger;

pub fn generate_system_model(file_path: &str, simulator: Arc<dyn SystemSimulator>) -> Result<Clients> {
    logger::init();
    log::info!("Logger initialized. Starting SystemModel construction.");

    let root_dto: ClientsDto = parse_json_file::<ClientsDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let system_model = Clients::from_dto(root_dto, simulator)?;
    log::info!("Internal SystemModel constructed successfully.");

    Ok(system_model)
}
