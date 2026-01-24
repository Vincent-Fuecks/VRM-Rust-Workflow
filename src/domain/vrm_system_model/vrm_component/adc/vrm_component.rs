use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationTrait};
use crate::domain::vrm_system_model::vrm_component::adc::ADC;
use crate::domain::vrm_system_model::vrm_component::component_communication::protocol::{Envelope, Payload};
use crate::domain::vrm_system_model::vrm_component::{
    utils::{vrm_component_base::VrmComponentTyp, vrm_component_message::VrmComponentMessage},
    vrm_component_trait::VrmComponent,
};
use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Handler};
use rand::prelude::IndexedRandom;

impl VrmComponent for ADC {
    fn get_typ(&self) -> VrmComponentTyp {
        self.base.typ
    }

    fn commit<A>(&mut self, res: Reservation, _ctx: &mut A::Context)
    where
        A: Actor + Handler<VrmComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, VrmComponentMessage> + ToEnvelope<A, Envelope>,
    {
        // I am an ADC (Distributor)
        // Pick a random child to forward the reservation to
        let mut rng = rand::rng();
        let keys: Vec<&String> = self.base.children.keys().collect();

        if let Some(&random_child_id) = keys.choose(&mut rng) {
            if let Some(child_addr) = self.base.children.get(random_child_id) {
                log::info!("[{}] ADC: Forwarding reservation {} to random child {}", self.base.id, res.get_name(), random_child_id);
                child_addr.do_send(Envelope {
                    target_id: random_child_id.clone(),
                    sender_id: self.base.id.clone(),
                    payload: Payload::Commit { reservation: res },
                });
            }
        } else {
            log::warn!("[{}] ADC: No children available to commit reservation", self.base.id);
        }
    }
}
