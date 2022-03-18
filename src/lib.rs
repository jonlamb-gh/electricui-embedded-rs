#![no_std]
//#![deny(warnings, clippy::all)]

// TODO
// - support offset / split packets
// - static assertions
// - error types
// - support partial payloads/metadata
// - split up the types into mods
// - add the send APIs and others
// - tests

// TODO - add a prelude file instead
pub use crate::decoder::Decoder;
pub use crate::error::Error;
pub use crate::message::{MessageId, MessageType};

pub mod decoder;
mod error;
mod message;
mod sealed;
pub mod wire;
