use anyhow::{Context, Ok, Result};

use crate::domain::vrm_system_model::rms::slurm::{
    payload::task_properties::TaskSubmission,
    response::{delete::SlurmDeleteResponse, nodes::SlurmNodesResponse, task_submit::TaskSubmitResponse, tasks::SlurmTaskResponse},
    rms_trait::SlurmRestApi,
    slurm_endpoint::SlurmEndpoint,
    slurm_rest_client::SlurmRestApiClient,
};

/// Implementation of the [`SlurmRestApi`] trait for the [`SlurmRestApiClient`].
///
/// This implementation handles the low-level HTTP communication, URL construction
/// via [`SlurmEndpoint`], and error handling specific to Slurm's REST response schemas.
#[async_trait::async_trait]
impl SlurmRestApi for SlurmRestApiClient {
    async fn get_nodes(&self) -> Result<SlurmNodesResponse> {
        let res = self.client.get(self.url(&SlurmEndpoint::Nodes.path())).send().await.context("Failed to send node request to Slurm")?;

        return res.error_for_status()?.json().await.context("Failed to parse SlurmNodesResponse");
    }

    async fn get_tasks(&self) -> Result<SlurmTaskResponse> {
        let res = self.client.get(self.url(&SlurmEndpoint::Jobs.path())).send().await.context("Failed to send jobs request to Slurm")?;

        return res.error_for_status()?.json().await.context("Failed to parse SlurmTaskResponse");
    }

    async fn is_rms_alive(&self) -> Result<bool> {
        let res = self.client.get(self.url(&SlurmEndpoint::Ping.path())).send().await.context("Failed to reach Slurm REST API for ping")?;

        return Ok(res.status().is_success());
    }

    async fn commit(&self, payload: TaskSubmission) -> Result<u32> {
        let res = self
            .client
            .post(self.url(&SlurmEndpoint::JobSubmit.path()))
            .json(&payload)
            .send()
            .await
            .context("Failed to connect to Slurm REST API during submission.")?;

        if !res.status().is_success() {
            let status = res.status();
            let err_body = res.text().await.unwrap_or_else(|_| "Could not read error body".to_string());
            anyhow::bail!("Slurm job submission failed [Status {}]: {}", status, err_body);
        }

        let response_data: TaskSubmitResponse = res.json().await.context("Failed to parse submission response")?;

        // Return the job id for meta-scheduler logic
        return response_data
            .job_id
            .ok_or_else(|| anyhow::anyhow!("Slurm accepted the job but didn't return an Id: Error context: {:?}", response_data.error));
    }

    async fn delete(&self, task_id: u32) -> Result<bool> {
        let path = format!("{}/{}", SlurmEndpoint::Job.path(), task_id);

        let res = self.client.delete(self.url(&path)).send().await.context(format!("Failed to send delete request for task {}", task_id))?;

        let response_data: SlurmDeleteResponse = res.json().await.context("Failed to parse delete response")?;

        if let Some(errors) = response_data.errors {
            if !errors.is_empty() {
                anyhow::bail!("Slurm encountered an error in the deletion process: {:?}", errors);
            }
        }

        return Ok(true);
    }
}
