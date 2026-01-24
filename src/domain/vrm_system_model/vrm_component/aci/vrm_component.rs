use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationTrait};
use crate::domain::vrm_system_model::vrm_component::aci::AcI;
use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::Envelope;
use crate::domain::vrm_system_model::vrm_component::{
    utils::{vrm_component_base::VrmComponentTyp, vrm_component_message::VrmComponentMessage},
    vrm_component_trait::VrmComponent,
};
use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Handler};

impl VrmComponent for AcI {
    fn get_typ(&self) -> VrmComponentTyp {
        self.base.typ
    }

    fn commit<A>(&mut self, res: Reservation, _ctx: &mut A::Context)
    where
        A: Actor + Handler<VrmComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, VrmComponentMessage> + ToEnvelope<A, Envelope>,
    {
        log::info!("[AcI {}] Processing reservation: {}", self.base.id, res.get_name());
    }
}
