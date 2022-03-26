#![no_std]
#![deny(warnings, clippy::all)]

// TODO
// - support offset / split packets
// - static assertions
// - error types
// - support partial payloads/metadata
// - add the send APIs and others
// - tests

pub mod decoder;
mod error;
mod message;
pub mod prelude;
mod sealed;
pub mod wire;
