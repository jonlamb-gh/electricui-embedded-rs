#![no_main]
#![deny(warnings, clippy::all)]

use electricui_embedded::prelude::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let p = match Packet::new(data) {
        Ok(p) => p,
        Err(_) => return,
    };
    let _ = p.data_length();
    let _ = p.typ();
    let _ = p.internal();
    let _ = p.offset();
    let _ = p.id_length();
    let _ = p.response();
    let _ = p.acknum();
    let _ = p.msg_id();
    let _ = p.msg_id_raw();
    let _ = p.payload();
    let _ = p.checksum();
    let _ = p.compute_checksum();
    let _ = p.wire_size();
});
