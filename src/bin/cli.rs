use core::str;

use bytes::BytesMut;
use clap::{arg, ArgAction, Parser};
use redis::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Parser, Debug)]
#[command(version, disable_help_flag = true)]
struct Args {
    #[arg(long, action = ArgAction::Help)]
    help: (),
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
    #[arg(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() -> redis::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let client = Client::connect((args.host, args.port)).await?;

    Ok(())
}
