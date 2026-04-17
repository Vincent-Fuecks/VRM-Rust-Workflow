//////////////////////////////////////////////////////////
/// File contains all constants of the VRM-Rust system ///
//////////////////////////////////////////////////////////
use crate::api::rms_config_dto::rms_dto::SlurmConfigDto;

/// Specifies the time interval, in which the ResourceStore and the Schedule of the
/// corresponding Slurm Rms system is synchronized regarding nodes and tasks.
pub const SCHEDULE_SYNC_TIMEINTERVAL_S: u64 = 60;

/// Specifies the memory each task on the slurm cluster receives.
/// In a later implementation, this should be handled differently.
pub const MEMORY_PER_NODE: u32 = 512;

/// Defines the duration the VRM waits for the response of a commit request to a local Rms.
pub const SLURM_RMS_COMMIT_TIMEOUT_S: u64 = 5;

/// Defines the duration the VRM waits for the response of a delete request to a local Rms.
pub const SLURM_RMS_DELETE_TIMEOUT_S: u64 = 5;

pub const SLURM_TEST_BASE_URL: &str = "http://localhost:6820";
pub const SLURM_TEST_VERSION: &str = "v0.0.41";
pub const SLURM_TEST_JWT_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjIwOTE3ODQyMzUsImlhdCI6MTc3NjQyNDIzNSwic3VuIjoicm9vdCJ9.oCbBkp1U6qTKGBhCd_Y57QupAN-laOBbGFFCwWUi2Qs";
pub const SLURM_TEST_USER_NAME: &str = "root";
