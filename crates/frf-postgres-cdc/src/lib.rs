#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod consumer;
pub mod decode;

pub use config::CdcConfig;
pub use consumer::PostgresCdcConsumer;
