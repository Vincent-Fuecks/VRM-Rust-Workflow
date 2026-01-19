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
use std::collections::HashMap;

pub struct AcI {
    pub base: GridComponentBase,
}

impl Actor for AcI {
    type Context = Context<Self>;
}

impl AcI {
    pub fn new(id: String) -> Self {
        let base = GridComponentBase::new(id, HashMap::new(), None, GridComponentTyp::AcI);

        Self { base }
    }
}

impl GridComponent for AcI {
    fn get_typ(&self) -> GridComponentTyp {
        self.base.typ
    }

    fn commit<A>(&mut self, res: Reservation, _ctx: &mut A::Context)
    where
        A: Actor + Handler<GridComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, GridComponentMessage> + ToEnvelope<A, Envelope>,
    {
        log::info!("[AcI {}] Processing reservation: {}", self.base.id, res.get_name());
    }
}

impl Handler<GridComponentMessage> for AcI {
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

impl Handler<Envelope> for AcI {
    type Result = ();
    fn handle(&mut self, msg: Envelope, ctx: &mut Self::Context) {
        self.handle(GridComponentMessage::Route(msg), ctx);
    }
}
