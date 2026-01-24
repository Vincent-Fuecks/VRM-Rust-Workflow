use crate::domain::vrm_system_model::vrm_component::adc::ADC;
use actix::prelude::{Actor, Context};

impl Actor for ADC {
    type Context = Context<Self>;
}
