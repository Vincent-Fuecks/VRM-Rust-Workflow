use anyhow::Result;
use reqwest::{Client, header};

use crate::api::rms_config_dto::rms_dto::SlurmConfigDto;

#[derive(Debug, Clone)]
pub struct SlurmConfig {
    pub base_url: String,
    pub version: String,
    pub user_name: String,
    pub jwt_token: String,
}

#[derive(Debug, Clone)]
pub struct SlurmRestApiClient {
    pub client: Client,
    config: SlurmConfig,
}

impl SlurmRestApiClient {
    pub fn new(slurm_config_dto: SlurmConfigDto) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("X-SLURM-USER-NAME", header::HeaderValue::from_str(&slurm_config_dto.user_name)?);
        headers.insert("X-SLURM-USER-TOKEN", header::HeaderValue::from_str(&slurm_config_dto.jwt_token)?);
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

        let client = Client::builder().default_headers(headers).build()?;

        let config = SlurmConfig {
            base_url: slurm_config_dto.base_url,
            version: slurm_config_dto.version,
            user_name: slurm_config_dto.user_name,
            jwt_token: slurm_config_dto.jwt_token,
        };

        Ok(Self { client, config })
    }

    // Helper to build the full URL: base + version + endpoint
    pub fn url(&self, endpoint: &str) -> String {
        format!("{}/slurm/{}{}", self.config.base_url, self.config.version, endpoint)
    }
}
