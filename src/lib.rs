pub mod codec;
pub mod command;
pub mod connection;
pub mod parser;
pub mod server;
pub mod value;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
