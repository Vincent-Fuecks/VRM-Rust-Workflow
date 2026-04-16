use anyhow::{Context, Ok, Result};

use crate::domain::vrm_system_model::rms::slurm::{
    payload::task_properties::TaskSubmission,
    response::{delete::SlurmDeleteResponse, nodes::SlurmNodesResponse, task_submit::TaskSubmitResponse, tasks::SlurmTaskResponse},
    rms_trait::SlurmRestApi,
    slurm_endpoint::SlurmEndpoint,
    slurm_rest_client::SlurmRestApiClient,
};

#[async_trait::async_trait]
impl SlurmRestApi for SlurmRestApiClient {
    // GET /slurm/v0.0.40/config
    async fn init_rms(&self) -> Result<bool> {
        // Mocking a successful initialization
        Ok(true)
    }

    // GET /slurm/v0.0.40/nodes
    async fn get_nodes(&self) -> Result<SlurmNodesResponse> {
        let res = self.client.get(self.url(&SlurmEndpoint::Nodes.path())).send().await.context("Failed to send node request to Slurm")?;

        let nodes: SlurmNodesResponse = res.json().await?;
        return Ok(nodes);
    }

    // "http://localhost:6820/slurm/v0.0.41/jobs"
    async fn get_tasks(&self) -> Result<SlurmTaskResponse> {
        let res = self.client.get(self.url(&SlurmEndpoint::Jobs.path())).send().await.context("Failed to send jobs request to Slurm")?;

        let tasks: SlurmTaskResponse = res.json().await?;
        return Ok(tasks);
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
        let res = self.client.get(self.url(&SlurmEndpoint::Ping.path())).send().await.context("Failed to send ping request to Slurm")?;
        Ok(res.status().is_success())
    }

    // GET /slurm/v0.0.40/diag
    async fn get_diagnostics(&self) -> Result<bool> {
        Ok(true)
    }

    // POST /slurm/v0.0.40/job/submit
    async fn commit(&self, payload: TaskSubmission) -> Result<u32> {
        let res = self
            .client
            .post(self.url(&SlurmEndpoint::JobSubmit.path()))
            .json(&payload)
            .send()
            .await
            .context("Failed to connect to Slurm REST API")?;

        if !res.status().is_success() {
            let err_body = res.text().await.unwrap_or_default();
            anyhow::bail!("Slurm job submission failed: {}", err_body);
        }

        let response_data: TaskSubmitResponse = res.json().await.context("Failed to parse submission response")?;

        // Return the Job id for meta-scheduler
        response_data.job_id.ok_or_else(|| anyhow::anyhow!("Slurm accepted the job but didn't return an ID: {:?}", response_data.error))
    }

    // DELETE /slurm/v0.0.40/job/{job_id}
    async fn delete(&self, task_id: u32) -> Result<bool> {
        let path = format!("{}/{}", SlurmEndpoint::Job.path(), task_id);

        let res = self.client.delete(self.url(&path)).send().await.context("Failed to send delete request to Slurm")?;

        let response_data: SlurmDeleteResponse = res.json().await.context("Failed to parse delete response")?;

        if let Some(errors) = response_data.errors {
            if !errors.is_empty() {
                anyhow::bail!("Slurm encountered an error in the deletion process: {:?}", errors);
            }
        }

        return Ok(true);
    }
}
