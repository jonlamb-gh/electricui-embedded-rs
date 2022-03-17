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

// TODO - add a prelude file

pub use crate::error::Error;
pub use crate::message::{MessageId, MessageType};

mod error;
mod message;
pub mod wire;
