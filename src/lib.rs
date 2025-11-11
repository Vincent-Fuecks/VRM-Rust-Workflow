use crate::api::client_dto::SystemModelDto;
use crate::domain::client::SystemModel;
use crate::loader::parser::parse_json_file;
use crate::error::Result;

pub mod api;
pub mod domain;
pub mod loader;
pub mod logger;
pub mod error;

pub fn generate_system_model(file_path: &str) -> Result<SystemModel> {
    logger::init();
    log::info!("Logger initialized. Starting SystemModel construction.");

    let root_dto: SystemModelDto = parse_json_file::<SystemModelDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let system_model = SystemModel::from_dto(root_dto)?;
    log::info!("Internal SystemModel constructed successfully.");

    Ok(system_model)
}