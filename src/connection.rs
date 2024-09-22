use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt as _, BufWriter},
    net::TcpStream,
};

use crate::{command::Command, value::Value};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }
}
