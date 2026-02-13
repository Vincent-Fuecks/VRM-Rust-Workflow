#[derive(Debug)]
pub enum SlurmEndpoint {
    Nodes,
    Jobs,
    Config,
}

impl SlurmEndpoint {
    pub fn path(&self) -> &str {
        match self {
            Self::Nodes => "/nodes",
            Self::Jobs => "/jobs",
            Self::Config => "/config",
        }
    }

    // TODO
    // You can even include the HTTP method
    // pub fn method(&self) -> reqwest::Method {
    //     match self {
    //         Self::JobSubmit => reqwest::Method::POST,
    //         _ => reqwest::Method::GET,
    //     }
    // }
}
