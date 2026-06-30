#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod apply;
pub mod error;
pub mod merge;
pub mod store;

pub use apply::LoroDeltaApplier;
pub use error::CrdtError;
pub use merge::apply_delta;
pub use store::{InMemoryCrdtStore, export_updates_since, merge_into_store};
