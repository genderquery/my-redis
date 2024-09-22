use std::net::IpAddr;

use tokio::net::TcpListener;
use tracing::trace;

use crate::connection::Connection;

pub const DEFAULT_BIND: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 6379;

#[derive(Debug)]
pub struct Config {
    pub bind: IpAddr,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND.parse().unwrap(),
            port: DEFAULT_PORT,
        }
    }
}

pub async fn run(config: Config) -> crate::Result<()> {
    let socket = TcpListener::bind((config.bind, config.port)).await?;

    loop {
        let (stream, addr) = socket.accept().await?;
        trace!("accepted connection from {}", addr);

        let connection = Connection::new(stream);
    }

    Ok(())
}
