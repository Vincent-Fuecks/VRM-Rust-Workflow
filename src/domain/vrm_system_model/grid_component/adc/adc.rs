use crate::domain::vrm_system_model::grid_component::component_communication::protocol::{Envelope, Payload};
use crate::domain::vrm_system_model::grid_component::{
    grid_component_trait::GridComponent,
    utils::{
        grid_component_base::{GridComponentBase, GridComponentTyp},
        grid_component_message::GridComponentMessage,
    },
};
use crate::domain::vrm_system_model::reservation::reservation::{Reservation, ReservationTrait};
use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Context, Handler};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use std::collections::HashMap;

pub struct ADC {
    pub base: GridComponentBase,
}

impl Actor for ADC {
    type Context = Context<Self>;
}

impl ADC {
    pub fn new(id: String) -> Self {
        let base = GridComponentBase::new(id, HashMap::new(), None, GridComponentTyp::ADC);

        Self { base }
    }
}

impl GridComponent for ADC {
    fn get_typ(&self) -> GridComponentTyp {
        self.base.typ
    }

    fn commit<A>(&mut self, res: Reservation, _ctx: &mut A::Context)
    where
        A: Actor + Handler<GridComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, GridComponentMessage> + ToEnvelope<A, Envelope>,
    {
        // I am an ADC (Distributor)
        // Pick a random child to forward the reservation to
        let mut rng = rand::thread_rng();
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

impl Handler<GridComponentMessage> for ADC {
    type Result = ();
    fn handle(&mut self, msg: GridComponentMessage, ctx: &mut Self::Context) {
        match msg {
            GridComponentMessage::RegisterChild { id, addr } => {
                self.base.children.insert(id, addr);
            }
            GridComponentMessage::Disconnect { id } => {
                self.base.children.remove(&id);
            }
            GridComponentMessage::SetParent(addr) => {
                self.base.parent = Some(addr);
                if let Some(p) = &self.base.parent {
                    p.do_send(Envelope::handshake(self.base.id.clone(), "parent".into()));
                }
            }
            GridComponentMessage::Route(env) => {
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

impl Handler<Envelope> for ADC {
    type Result = ();
    fn handle(&mut self, msg: Envelope, ctx: &mut Self::Context) {
        self.handle(GridComponentMessage::Route(msg), ctx);
    }
}
