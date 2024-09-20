pub mod client;
pub mod connection;
pub mod parser;

pub use client::Client;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
