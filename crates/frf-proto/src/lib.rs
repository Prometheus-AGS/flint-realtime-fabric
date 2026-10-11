#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::pedantic)]

// Generated message types from proto/flint/v1/*.proto — do not hand-edit.
// Re-run `cargo build -p frf-proto` to regenerate after proto changes.

pub mod flint {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/flint.v1.rs"));
    }
    pub mod v2 {
        include!(concat!(env!("OUT_DIR"), "/flint.v2.rs"));
    }
}

pub use flint::v1 as fv1;
pub use flint::v2 as fv2;
