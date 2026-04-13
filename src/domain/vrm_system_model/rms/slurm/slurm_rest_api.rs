use anyhow::{Context, Ok, Result};
use reqwest::Client;

use crate::domain::vrm_system_model::rms::slurm::{rms_trait::SlurmRestApi, slurm::SlurmRms, slurm_endpoint::SlurmEndpoint, slurm_rest_client::SlurmRestApiClient};

#[async_trait::async_trait]
impl SlurmRestApi for SlurmRestApiClient {
    // GET /slurm/v0.0.40/config
    async fn init_rms(&self) -> Result<bool> {
        // Mocking a successful initialization
        Ok(true)
    }

    // GET /slurm/v0.0.40/nodes
    async fn sync_nodes(&self) -> Result<bool> {
        // In a real mock, you might update an internal state here
        Ok(true)
    }

    // "http://localhost:6820/slurm/v0.0.40/jobs"
    async fn sync_tasks(&self) -> Result<bool> {
        Ok(true)
    }

    // Only Pending Jobs: /slurm/v0.0.40/jobs?state=PENDING
    async fn get_waiting_task_for_execution(&self) -> Result<bool> {
        Ok(true)
    }

    // POST /slurm/v0.0.40/node/{name}
    async fn update_node_status(&self) -> Result<bool> {
        Ok(true)
    }

    // GET /slurm/v0.0.40/ping
    async fn is_rms_alive(&self) -> Result<bool> {
        let res = self.client
            .get(self.url(&SlurmEndpoint::Ping.path()))
            .send()
            .await
            .context("Failed to send ping request to Slurm")?;
        Ok(res.status().is_success())
    }

    // GET /slurm/v0.0.40/diag
    async fn get_diagnostics(&self) -> Result<bool> {
        Ok(true)
    }

    // POST /slurm/v0.0.40/job/submit
    async fn commit(&self, _client: &Client) -> Result<bool> {
        // Note: Using _client to ignore the unused parameter warning
        Ok(true)
    }

    // DELETE /slurm/v0.0.40/job/{job_id}
    async fn delete(&self) -> Result<bool> {
        Ok(true)
    }
}