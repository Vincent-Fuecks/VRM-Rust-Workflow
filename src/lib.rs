use crate::api::vrm_system_model_dto::vrm_dto::VrmSystemModelDto;
use crate::api::workflow_dto::client_dto::SystemModelDto;
use crate::domain::vrm_system_model::client::client::SystemModel;
use crate::domain::vrm_system_model::vrm_system_model::VrmSystemModel;
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

pub fn generate_vrm_model(file_path: &str) -> Result<VrmSystemModel> {
    logger::init();
    log::info!("Logger initialized. Starting VrmSystemModel construction.");

    let root_dto: VrmSystemModelDto = parse_json_file::<VrmSystemModelDto>(file_path)?;
    log::info!("JSON file parsed successfully.");

    let vrm_system_model = VrmSystemModel::from_dto(root_dto)?;
    log::info!("Internal SystemModel constructed successfully.");

    Ok(vrm_system_model)
}
