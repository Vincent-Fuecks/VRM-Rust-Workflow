use std::fmt::Debug;

use anyhow::Result;

use crate::domain::vrm_system_model::rms::slurm::{payload::task_properties::{TaskProperties, TaskSubmission}, response::{nodes::SlurmNodesResponse, tasks::{SlurmTask, SlurmTaskResponse}}};

#[async_trait::async_trait]
pub trait SlurmRestApi: Debug {
    // GET /slurm/v0.0.40/config
    async fn init_rms(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/nodes
    async fn get_nodes(&self) -> Result<SlurmNodesResponse>;

    // "http://localhost:6820/slurm/v0.0.40/jobs"
    async fn get_tasks(&self) -> Result<SlurmTaskResponse>;

    // Only Pending Jobs: /slurm/v0.0.40/jobs?state=PENDING
    async fn get_waiting_task_for_execution(&self) -> Result<bool>;

    // POST /slurm/v0.0.40/node/{name}
    async fn update_node_status(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/ping
    async fn is_rms_alive(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/diag
    async fn get_diagnostics(&self) -> Result<bool>;

    // POST /slurm/v0.0.40/job/submit
    async fn commit(&self, payload: TaskSubmission) -> Result<u32>;

    // DELETE /slurm/v0.0.40/job/{job_id}
    async fn delete(&self, task_id: u32) -> Result<bool>;
}
