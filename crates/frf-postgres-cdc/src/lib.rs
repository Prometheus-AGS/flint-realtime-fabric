#![deny(warnings)]
#![warn(clippy::pedantic)]

mod canonical;
mod catalog;
mod catalog_validation;
pub mod config;
pub mod consumer;
pub mod decode;
pub mod model;
mod transaction;

#[cfg(test)]
mod transaction_tests;

pub use config::{CdcConfig, ConfigError, TableEnrollment, TenantMode};
pub use consumer::PostgresCdcConsumer;
