#![no_std]
#![deny(warnings, clippy::all)]

// TODO
// - support offset / split packets
// - static assertions
// - error types
// - support partial payloads/metadata
// - add the send APIs and others
// - tests

pub use crate::error::Error;

pub mod decoder;
pub mod error;
pub mod message;
pub mod prelude;
mod sealed;
pub mod wire;
