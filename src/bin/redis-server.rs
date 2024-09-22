use const_format::formatcp;
use pico_args::Arguments;
use redis::server::{Config, DEFAULT_BIND, DEFAULT_PORT};

const VERSION: &str = concat!("redis-server ", env!("CARGO_PKG_VERSION"));
const HELP: &str = formatcp!(
    "\
{VERSION}

Usage: redis-server [OPTIONS]
    -h, --help  Output this message and exit.
    --version   Out version and exit.
    --bind      Address to bind to (default: {DEFAULT_BIND}).
    --port      Accept connection on port (default: {DEFAULT_PORT}).
"
);

fn parse_args() -> Result<Config, pico_args::Error> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    if args.contains("--version") {
        print!("{}", VERSION);
        std::process::exit(0);
    }

    let args = Config {
        bind: args
            .opt_value_from_str("--bind")?
            .unwrap_or(DEFAULT_BIND.parse().unwrap()),
        port: args.opt_value_from_str("--port")?.unwrap_or(DEFAULT_PORT),
    };

    Ok(args)
}

#[tokio::main]
async fn main() -> redis::Result<()> {
    tracing_subscriber::fmt::init();

    let args = parse_args()?;

    redis::server::run(args).await?;

    Ok(())
}
