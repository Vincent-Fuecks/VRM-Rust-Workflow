use crate::domain::vrm_system_model::vrm_component::aci::AcI;
use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::{Envelope, Payload};
use crate::domain::vrm_system_model::vrm_component::{utils::vrm_component_message::VrmComponentMessage, vrm_component_trait::VrmComponent};

use actix::prelude::Handler;

impl Handler<VrmComponentMessage> for AcI {
    type Result = ();
    fn handle(&mut self, msg: VrmComponentMessage, ctx: &mut Self::Context) {
        match msg {
            VrmComponentMessage::RegisterChild { id, addr } => {
                self.base.children.insert(id, addr);
            }
            VrmComponentMessage::Disconnect { id } => {
                self.base.children.remove(&id);
            }
            VrmComponentMessage::SetParent(addr) => {
                self.base.parent = Some(addr);
                if let Some(p) = &self.base.parent {
                    p.do_send(Envelope::handshake(self.base.id.clone(), "parent".into()));
                }
            }
            VrmComponentMessage::Route(env) => {
                if env.target_id == self.base.id {
                    if let Payload::Commit { reservation } = env.payload {
                        self.commit::<Self>(reservation, ctx);
                        return;
                    }
                }
                self.base.handle_route(env, ctx);
            }
        }
    }
}

impl Handler<Envelope> for AcI {
    type Result = ();
    fn handle(&mut self, msg: Envelope, ctx: &mut Self::Context) {
        self.handle(VrmComponentMessage::Route(msg), ctx);
    }
}
