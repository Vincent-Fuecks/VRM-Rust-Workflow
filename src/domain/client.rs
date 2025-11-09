use std::collections::HashMap;
use std::collections::hash_map::Entry; 
use crate::api::workflow_dto::{RootDto, ClientDto};
use crate::domain::workflow::{Workflow, Task};
use log::{info, error, debug};

#[derive(Debug)]
pub struct Client {
    pub id: String, 
    pub workflows: Vec<Workflow>,
}

#[derive(Debug)]
pub struct ClientRegistry {
    clients: HashMap<String, Client>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        ClientRegistry {
            clients: HashMap::new(),
        }
    }

    pub fn update_clients(&mut self, json_str: &str) {
        let root: RootDto = match serde_json::from_str(json_str) {
            Ok(r) => r,
            Err(e) => {
                // --- Use the logger ---
                error!("Failed to parse workflow JSON: {}", e);
                return;
            }
        };

        debug!("JSON parsed successfully. Processing {} clients.", root.clients.len());

        // Iterate over the DTO clients
        for client_dto in root.clients {
            match self.clients.entry(client_dto.id.clone()) {
                
                // Case 1: The client is already in our registry
                Entry::Occupied(mut entry) => {
                    let existing_client = entry.get_mut();
                    // --- Use the logger ---
                    info!(
                        "Client '{}' already exists. Merging {} new workflows.",
                        existing_client.id,
                        client_dto.workflows.len()
                    );

                    let new_workflows = client_dto.workflows.into_iter().map(Workflow::from);
                    existing_client.workflows.extend(new_workflows);
                }
                
                // Case 2: This is a new client
                Entry::Vacant(entry) => {
                    info!(
                        "Adding new client '{}' with {} workflows.",
                        client_dto.id,
                        client_dto.workflows.len()
                    );

                    let new_client = Client {
                        id: client_dto.id, // Use the ID from the DTO
                        workflows: client_dto.workflows.into_iter().map(Workflow::from).collect(),
                    };

                    entry.insert(new_client);
                }
            }
        }
    }

    /// A helper function to print the current state of the registry
    pub fn print_summary(&self) {
        info!("--- Registry Summary ---");
        info!("Total unique clients: {}", self.clients.len());

        for (id, client) in &self.clients {
            info!("  - Client ID: {}", id);
            info!("    Workflows: {}", client.workflows.len());

            for (i, workflow) in client.workflows.iter().enumerate() {
                info!("      * Workflow [{}]: {}", i, workflow.name);

                for (j, task) in workflow.tasks.iter().enumerate() {
                    debug!("        - Task [{}]: {} (ID: {})", j, task.name, task.id);
                }
            }
        }
        info!("------------------------");
    }
}