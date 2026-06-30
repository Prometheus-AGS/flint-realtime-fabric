#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod error;
pub mod sfu;

pub use error::StrOmError;
pub use sfu::StrOmSignaler;
