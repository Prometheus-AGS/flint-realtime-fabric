#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod error;
mod model;
mod store;

pub use error::SurrealProjectionError;
pub use store::SurrealEntityProjection;
