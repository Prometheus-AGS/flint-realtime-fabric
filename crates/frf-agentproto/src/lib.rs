#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod content_block;
pub mod convert;
pub mod error;

pub use content_block::ContentBlock;
pub use convert::{domain_from_proto, domain_to_proto};
pub use error::AgentProtoError;
