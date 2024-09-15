use clap::{arg, Parser};
use tokio::net::TcpStream;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
    #[arg(short, long, default_value = "6379")]
    port: u16,
}

#[tokio::main]
async fn main() -> redis::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let stream = TcpStream::connect((args.host, args.port)).await?;

    Ok(())
}
