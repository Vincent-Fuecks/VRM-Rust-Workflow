use std::fmt::Debug;

use anyhow::Result;
use reqwest::Client;

#[async_trait::async_trait]
pub trait SlurmRestApi: Debug {
    // GET /slurm/v0.0.40/config
    async fn init_rms(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/nodes
    async fn sync_nodes(&self) -> Result<bool>;

    // "http://localhost:6820/slurm/v0.0.40/jobs"
    async fn sync_tasks(&self) -> Result<bool>;

    // Only Pending Jobs: /slurm/v0.0.40/jobs?state=PENDING
    async fn get_waiting_task_for_execution(&self) -> Result<bool>;

    // POST /slurm/v0.0.40/node/{name}
    async fn update_node_status(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/ping
    async fn is_rms_alive(&self) -> Result<bool>;

    // GET /slurm/v0.0.40/diag
    async fn get_diagnostics(&self) -> Result<bool>;

    // POST /slurm/v0.0.40/job/submit
    async fn commit(&self, client: &Client) -> Result<bool>;

    // DELETE /slurm/v0.0.40/job/{job_id}
    async fn delete(&self) -> Result<bool>;
}
