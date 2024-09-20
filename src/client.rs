use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{connection::Connection, Result};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);
        Ok(Client { connection })
    }
}
