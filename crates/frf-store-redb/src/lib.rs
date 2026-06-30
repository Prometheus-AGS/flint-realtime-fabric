#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
mod key;
pub mod store;

pub use error::RedbOpStoreError;
pub use store::RedbOpStore;
