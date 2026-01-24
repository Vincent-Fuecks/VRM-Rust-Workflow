use actix::prelude::{Message, Recipient};

use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::Envelope;

#[derive(Message)]
#[rtype(result = "()")]
pub enum VrmComponentMessage {
    RegisterChild { id: String, addr: Recipient<Envelope> },
    Disconnect { id: String },
    Route(Envelope),
    SetParent(Recipient<Envelope>),
}
