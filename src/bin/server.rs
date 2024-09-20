use core::str;
use std::io;

use clap::{arg, Parser};
use redis::{connection::Connection, parser::Value};
use tokio::net::TcpListener;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,
    #[arg(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() -> redis::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let listener = TcpListener::bind((args.bind, args.port)).await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut connection = Connection::new(socket);
            match connection.read().await {
                Ok(Some(value)) => {
                    println!("{:?}", value);
                    connection.write().await?;
                }
                Ok(None) => panic!("connection reset by peer"),
                Err(err) => panic!("{}", err),
            }
            anyhow::Ok(())
        });
    }
}
