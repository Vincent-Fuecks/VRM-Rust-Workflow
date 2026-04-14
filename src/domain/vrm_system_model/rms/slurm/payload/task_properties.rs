use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TaskSubmission {
    pub job: TaskProperties,
    pub script: String,
}

#[derive(Serialize, Debug)]
pub struct TaskProperties {
pub name: String,
    pub nodes: String,               
    pub cpus_per_task: u32,          
    pub current_working_directory: String,
    pub standard_output: String,
    pub environment: Vec<String>,    
}