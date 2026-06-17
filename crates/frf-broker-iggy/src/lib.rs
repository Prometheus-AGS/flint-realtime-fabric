#![deny(warnings)]
#![warn(clippy::pedantic)]

pub mod broker;
pub mod channel;
pub mod error;

pub use broker::IggyBroker;
