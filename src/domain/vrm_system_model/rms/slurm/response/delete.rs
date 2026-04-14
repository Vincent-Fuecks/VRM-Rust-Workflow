use serde::Deserialize;

use crate::domain::vrm_system_model::rms::slurm::response::nodes::{SlurmError, SlurmMeta, SlurmWarning};

#[derive(Debug, Deserialize)]
pub struct SlurmDeleteResponse {
    pub meta: Option<SlurmMeta>,
    pub errors: Option<Vec<SlurmError>>,
    pub warnings: Option<Vec<SlurmWarning>>,
}