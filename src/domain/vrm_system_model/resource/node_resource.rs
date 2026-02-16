use crate::domain::vrm_system_model::resource::{resource_trait::Resource, resources::BaseResource};
use crate::domain::vrm_system_model::utils::id::ResourceName;

use std::any::Any;

#[derive(Debug, Clone)]
pub struct NodeResource {
    pub base: BaseResource,
}

impl NodeResource {
    pub fn new(name: ResourceName, capacity: i64) -> Self {
        let base = BaseResource::new(name, capacity);
        Self { base }
    }
}

impl Resource for NodeResource {
    fn get_capacity(&self) -> i64 {
        self.base.capacity
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_name(&self) -> ResourceName {
        self.base.get_name()
    }
}
