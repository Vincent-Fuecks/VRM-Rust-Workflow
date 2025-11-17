use std::collections::HashMap;

use crate::api::workflow_dto::client_dto::{ClientDto, SystemModelDto};
use crate::domain::workflow::workflow::Workflow;
use crate::error::Result;

/// Represents a client, which can have multiple workflows.
#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub workflows: HashMap<String, Workflow>,
}

/// The root of the internal model, which can have multiple clients.
#[derive(Debug, Clone, Default)]
pub struct SystemModel {
    pub clients: HashMap<String, Client>,
}

impl SystemModel {
    pub fn from_dto(root_dto: SystemModelDto) -> Result<Self> {
        let mut clients = HashMap::new();

        for client_dto in root_dto.clients {
            let client = Client::from_dto(client_dto)?;
            clients.insert(client.id.clone(), client);
        }
        Ok(SystemModel { clients })
    }
}

impl Client {
    pub fn from_dto(dto: ClientDto) -> Result<Self> {
        let mut workflows = HashMap::new();

        for workflow_dto in dto.workflows {
            let workflow = Workflow::try_from(workflow_dto)?;
            workflows.insert(workflow.base.id.clone(), workflow);
        }
        Ok(Client {
            id: dto.id,
            workflows,
        })
    }
}
