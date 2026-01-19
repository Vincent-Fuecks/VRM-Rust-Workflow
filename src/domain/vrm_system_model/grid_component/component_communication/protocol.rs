use actix::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::vrm_system_model::reservation::reservation::Reservation;

/// The types of messages our system understands.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    /// Handshake: "Hi, I am Node X"
    Register { from_id: String },
    /// Command: "Do this work" (Downstream)
    Command { action: String },
    /// Data: "Here is the result" (Upstream)
    Data { sensor_value: f64 },
    /// Commit: A request to finalize a reservation
    Commit { reservation: Reservation },
}

/// The wrapper ensuring routing information accompanies every message.
#[derive(Serialize, Deserialize, Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct Envelope {
    pub target_id: String,
    pub sender_id: String,
    pub payload: Payload,
}

impl Envelope {
    pub fn handshake(my_id: String, parent_id: String) -> Self {
        Envelope { target_id: parent_id, sender_id: my_id.clone(), payload: Payload::Register { from_id: my_id } }
    }
}
