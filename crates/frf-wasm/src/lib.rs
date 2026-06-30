#![allow(clippy::missing_panics_doc)]

pub mod crdt;

// Web-only modules — only compiled for wasm32 target.
#[cfg(target_arch = "wasm32")]
pub mod agent;
#[cfg(target_arch = "wasm32")]
pub mod publish;
#[cfg(target_arch = "wasm32")]
pub mod subscribe;

#[cfg(target_arch = "wasm32")]
pub use agent::AgentStream;
pub use crdt::crdt_apply_delta;
