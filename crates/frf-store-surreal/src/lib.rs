#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod model;
pub mod store;

pub use error::SurrealStoreError;
pub use store::SurrealCrdtStore;
