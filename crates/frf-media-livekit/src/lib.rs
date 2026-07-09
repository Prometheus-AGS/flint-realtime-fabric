#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod adapter;
pub mod config;
pub mod error;
pub mod inbound;

pub use adapter::LiveKitSignaling;
pub use config::LiveKitConfig;
pub use error::LiveKitError;
pub use inbound::{LiveKitDataSource, spawn_inbound_relay};
