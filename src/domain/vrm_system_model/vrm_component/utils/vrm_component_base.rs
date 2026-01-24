use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;

use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::{Envelope, Payload};
use crate::domain::vrm_system_model::vrm_component::utils::vrm_component_message::VrmComponentMessage;

#[derive(Clone, Copy)]
pub enum VrmComponentTyp {
    ADC,
    AcI,
}

pub struct VrmComponentBase {
    pub id: String,
    pub children: HashMap<String, Recipient<Envelope>>,
    pub parent: Option<Recipient<Envelope>>,
    pub typ: VrmComponentTyp,
}

impl VrmComponentBase {
    pub fn new(id: String, children: HashMap<String, Recipient<Envelope>>, parent: Option<Recipient<Envelope>>, typ: VrmComponentTyp) -> Self {
        Self { id, children, parent, typ }
    }

    /// Common routing logic shared by components.
    /// Constrained A::Context to actix::Context<A> to satisfy trait bounds.
    pub fn handle_route<A>(&mut self, env: Envelope, _ctx: &mut Context<A>)
    where
        A: Actor<Context = Context<A>> + Handler<VrmComponentMessage>,
    {
        if env.target_id == self.id {
            match env.payload {
                Payload::Command { action } => log::info!("[{}] Cmd: {}", self.id, action),
                Payload::Data { sensor_value } => {
                    log::info!("[{}] Data: {}", self.id, sensor_value)
                }
                _ => {}
            }
        } else if let Some(child) = self.children.get(&env.target_id) {
            child.do_send(env);
        } else if let Some(parent) = &self.parent {
            parent.do_send(env);
        }
    }
}
