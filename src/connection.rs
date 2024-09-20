use std::error::Error;
use std::io;

use anyhow::bail;
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use winnow::error::ErrMode;
use winnow::{BStr, Partial};

use crate::parser::{self, Value};

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read(&mut self) -> anyhow::Result<Option<Value>> {
        loop {
            let mut input = Partial::new(BStr::new(&self.buffer));
            match parser::value(&mut input) {
                Ok(value) => return Ok(Some(value)),
                Err(ErrMode::Incomplete(_)) => (),
                Err(err) => bail!(err),
            };

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    bail!("connection reset by peer");
                }
            }
        }
    }

    pub async fn write(&mut self) -> anyhow::Result<()> {
        self.stream.write_all(b"+OK\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }
}
