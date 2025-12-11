use crate::domain::vrm_system_model::rms::rms::{Rms, RmsBase};
use std::any::Any;

#[derive(Debug)]
pub struct NullBroker {
    pub base: RmsBase,
}

impl Rms for NullBroker {
    fn get_base(&self) -> &RmsBase {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut RmsBase {
        &mut self.base
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
