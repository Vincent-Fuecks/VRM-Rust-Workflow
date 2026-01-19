use crate::domain::vrm_system_model::grid_component::{
    component_communication::codec::DistSystemCodec,
    component_communication::protocol::{Envelope, Payload},
    utils::grid_component_message::GridComponentMessage,
};
use actix::prelude::*;
use std::io;
use tokio::net::TcpStream;
use tokio_util::codec::FramedRead;

/// Represents a connection to a remote peer (Parent or Child).
/// It acts as a proxy: Messages sent to this actor are written to TCP.
/// Messages read from TCP are forwarded to the main Node actor.
pub struct TcpSession {
    /// Address of the main component actor to forward received messages to.
    /// Changed from Addr<Node> to Recipient<NodeMessage> for polymorphism.
    node: Recipient<GridComponentMessage>,
    /// Write sink for the TCP stream.
    /// GENERICS ORDER IS CRITICAL: <Item, IO, Codec>
    framed_write: actix::io::FramedWrite<Envelope, tokio::io::WriteHalf<TcpStream>, DistSystemCodec>,
    /// ID of the remote peer (discovered via Handshake)
    remote_id: Option<String>,
}

impl TcpSession {
    pub fn new(
        node: Recipient<GridComponentMessage>,
        write_half: tokio::io::WriteHalf<TcpStream>,
        read_half: tokio::io::ReadHalf<TcpStream>,
    ) -> Addr<Self> {
        Self::create(|ctx| {
            ctx.add_stream(FramedRead::new(read_half, DistSystemCodec::new()));
            Self { node, framed_write: actix::io::FramedWrite::new(write_half, DistSystemCodec::new(), ctx), remote_id: None }
        })
    }
}

impl Actor for TcpSession {
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(id) = self.remote_id.take() {
            self.node.do_send(GridComponentMessage::Disconnect { id });
        }
    }
}

impl Handler<Envelope> for TcpSession {
    type Result = ();

    fn handle(&mut self, msg: Envelope, _ctx: &mut Self::Context) {
        self.framed_write.write(msg);
    }
}

impl StreamHandler<Result<Envelope, io::Error>> for TcpSession {
    fn handle(&mut self, msg: Result<Envelope, io::Error>, ctx: &mut Self::Context) {
        match msg {
            Ok(env) => {
                if let Payload::Register { from_id } = &env.payload {
                    self.remote_id = Some(from_id.clone());
                    self.node.do_send(GridComponentMessage::RegisterChild { id: from_id.clone(), addr: ctx.address().recipient() });
                } else {
                    self.node.do_send(GridComponentMessage::Route(env));
                }
            }
            Err(e) => {
                log::error!("Codec error: {}", e);
                ctx.stop();
            }
        }
    }
}

impl actix::io::WriteHandler<io::Error> for TcpSession {}
