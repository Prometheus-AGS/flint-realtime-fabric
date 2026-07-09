#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod config;
pub mod demux;
pub mod driver;
pub mod error;
pub mod ice;
pub mod room;
pub mod rtc_spike;
pub mod session;
pub mod sfu;
pub mod transport_spike;

pub use config::MediaConfig;
pub use error::StrOmError;
pub use rtc_spike::{SpikeError, negotiate_answer};
pub use session::StrOmTransport;
pub use sfu::StrOmSignaler;
pub use transport_spike::{PollStep, TransportError, TransportLoop};
