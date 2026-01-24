use std::collections::HashMap;

use crate::domain::vrm_system_model::vrm_component::utils::vrm_component_base::{VrmComponentBase, VrmComponentTyp};

mod actor;
mod handler;
mod helpers;
mod vrm_component;

pub struct AcI {
    pub base: VrmComponentBase,
}

impl AcI {
    pub fn new(id: String) -> Self {
        let base = VrmComponentBase::new(id, HashMap::new(), None, VrmComponentTyp::AcI);

        Self { base }
    }
}
