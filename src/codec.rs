use tokio_util::codec::{Decoder, Encoder};

use crate::value::Value;

pub struct RespDecoder;

pub struct RespEncoder;

impl Decoder for RespDecoder {
    type Item = Value;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

impl Encoder<Value> for RespEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Value, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}
