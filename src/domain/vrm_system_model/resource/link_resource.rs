use std::any::Any;

use crate::domain::vrm_system_model::resource::resource_trait::Resource;
use crate::domain::vrm_system_model::resource::resources::BaseResource;
use crate::domain::vrm_system_model::schedule::slotted_schedule::slotted_schedule_context::SlottedScheduleContext;
use crate::domain::vrm_system_model::schedule::slotted_schedule::strategy::node::node_strategy::NodeStrategy;
use crate::domain::vrm_system_model::utils::id::{ResourceName, RouterId};

// TODO Naming is of should be just Link
#[derive(Debug, Clone)]
pub struct LinkResource {
    pub base: BaseResource,
    pub source: RouterId,
    pub target: RouterId,

    /// The schedule manages bandwidth for this link.
    pub schedule: SlottedScheduleContext<NodeStrategy>,
}

impl LinkResource {
    pub fn new(name: ResourceName, source: RouterId, target: RouterId, capacity: i64, schedule: SlottedScheduleContext<NodeStrategy>) -> Self {
        let base = BaseResource::new(name, capacity);

        Self { base, source, target, schedule }
    }
}

impl Resource for LinkResource {
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
