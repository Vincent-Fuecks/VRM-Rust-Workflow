use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Handler};

use crate::domain::vrm_system_model::reservation::reservation::Reservation;
use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::Envelope;
use crate::domain::vrm_system_model::vrm_component::utils::vrm_component_base::VrmComponentTyp;
use crate::domain::vrm_system_model::vrm_component::utils::vrm_component_message::VrmComponentMessage;

pub trait VrmComponent {
    fn get_typ(&self) -> VrmComponentTyp;
    fn commit<A>(&mut self, reservation: Reservation, ctx: &mut A::Context)
    where
        A: Actor + Handler<VrmComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, VrmComponentMessage> + ToEnvelope<A, Envelope>;
}
