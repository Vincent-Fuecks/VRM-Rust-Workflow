use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DependencyDto {
    pub pre: Vec<String>,
    pub sync: Vec<String>,
}