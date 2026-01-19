use crate::domain::vrm_system_model::grid_component::component_communication::protocol::{Envelope, Payload};
use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/// Combines LengthDelimitedCodec (TCP framing) with Bincode (Serialization).
pub struct DistSystemCodec {
    codec: LengthDelimitedCodec,
}

impl DistSystemCodec {
    pub fn new() -> Self {
        Self { codec: LengthDelimitedCodec::new() }
    }
}

impl Default for DistSystemCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder<Envelope> for DistSystemCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Envelope, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = bincode::serialize(&item).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let bytes = bytes::Bytes::from(bytes);
        self.codec.encode(bytes, dst)
    }
}

impl Decoder for DistSystemCodec {
    type Item = Envelope;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.codec.decode(src)? {
            Some(bytes) => {
                let item = bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }
}
