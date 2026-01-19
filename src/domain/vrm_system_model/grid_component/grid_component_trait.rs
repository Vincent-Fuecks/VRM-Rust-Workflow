use actix::dev::ToEnvelope;
use actix::prelude::{Actor, Handler};

use crate::domain::vrm_system_model::grid_component::component_communication::protocol::Envelope;
use crate::domain::vrm_system_model::grid_component::utils::grid_component_base::GridComponentTyp;
use crate::domain::vrm_system_model::grid_component::utils::grid_component_message::GridComponentMessage;
use crate::domain::vrm_system_model::reservation::reservation::Reservation;

/// The GridComponent trait defines the common behavior for all nodes in our grid.
pub trait GridComponent {
    fn get_typ(&self) -> GridComponentTyp;
    fn commit<A>(&mut self, reservation: Reservation, ctx: &mut A::Context)
    where
        // //Fixed Now: Added Handler bounds to trait definition so impls don't have "stricter requirements"
        A: Actor + Handler<GridComponentMessage> + Handler<Envelope>,
        A::Context: ToEnvelope<A, GridComponentMessage> + ToEnvelope<A, Envelope>;
}
