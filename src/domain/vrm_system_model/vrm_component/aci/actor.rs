use crate::domain::vrm_system_model::vrm_component::aci::AcI;

use actix::prelude::{Actor, Context};

impl Actor for AcI {
    type Context = Context<Self>;
}
