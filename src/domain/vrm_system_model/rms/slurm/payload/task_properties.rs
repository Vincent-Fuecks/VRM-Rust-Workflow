use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TaskSubmission {
    pub job: JobProperties,
    pub script: String,
}

#[derive(Serialize, Debug)]
pub struct JobProperties {
    pub name: String,
    pub cpus_per_task: u32,
    pub memory_per_node: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<String>,

    /// Earliest time the job can start (Unix Timestamp)
    pub begin: u64,

    /// Latest time the job must be finished (Unix Timestamp)
    pub deadline: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_working_directory: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_output: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard_error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
}
