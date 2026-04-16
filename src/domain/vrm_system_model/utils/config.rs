//////////////////////////////////////////////////////////
/// File contains all constants of the VRM-Rust system ///
//////////////////////////////////////////////////////////

/// Specifies the time interval, in which the ResourceStore and the Schedule of the
/// corresponding Slurm Rms system is synchronized regarding nodes and tasks.
pub const SCHEDULE_SYNC_TIMEINTERVAL_S: u64 = 60;

/// Specifies the memory each task on the slurm cluster receives.
/// In a later implementation, this should be handled differently.
pub const MEMORY_PER_NODE: u32 = 512;

/// Defines the duration the VRM waits for the response of a commit request to a local Rms.
pub const SLURM_RMS_COMMIT_TIMEOUT_S: u64 = 5;
