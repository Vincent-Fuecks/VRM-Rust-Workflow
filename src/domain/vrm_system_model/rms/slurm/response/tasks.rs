use std::str::FromStr;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SlurmTaskResponse {
    pub meta: Option<SlurmMeta>,
    pub errors: Option<Vec<SlurmError>>,
    pub warnings: Option<Vec<SlurmWarning>>,
    pub jobs: Vec<SlurmTask>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmTask {
    pub job_id: u32,
    pub name: Option<String>,
    pub job_state: Option<Vec<String>>,
    pub user_name: Option<String>,
    pub job_resources: Option<SlurmJobResources>,
    pub time: Option<SlurmTime>,
    pub command: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmJobResources {
    pub nodes: Option<String>,
    pub allocated_cpus: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmTime {
    /// Number of seconds the job has been running
    pub elapsed: Option<u64>,
    /// Time limit in minutes (Slurm default) or seconds depending on config
    pub limit: Option<u64>,
    /// Unix timestamp of actual or expected start
    pub start: Option<u64>,
    /// Unix timestamp of expected end (start + limit)
    pub end: Option<u64>,
    /// Unix timestamp of job submission
    pub submission: Option<u64>,
    /// Unix timestamp of when the job became eligible for scheduling
    pub eligible: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmMeta {
    pub plugin: Option<SlurmPlugin>,
    pub slurm: Option<SlurmVersionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmPlugin {
    pub r#type: String,
    pub name: String,
    pub data_parser: String,
}

#[derive(Debug, Deserialize)]
pub struct SlurmVersionInfo {
    pub release: String,
}

#[derive(Debug, Deserialize)]
pub struct SlurmError {
    pub error: String,
    pub error_number: i32,
    pub description: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlurmWarning {
    pub description: String,
    pub source: Option<String>,
}
