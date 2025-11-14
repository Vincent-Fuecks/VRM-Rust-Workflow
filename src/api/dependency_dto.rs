use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DependencyDto {
    pub data: Vec<String>,
    pub sync: Vec<String>,
}
