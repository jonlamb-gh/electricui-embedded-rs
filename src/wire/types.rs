use crate::message::MessageType;
use core::{cmp, mem};

impl MessageType {
    pub(crate) fn wire_size(self, num_elements: usize) -> usize {
        use MessageType::*;
        let cnt = cmp::max(1, num_elements);
        match self {
            Callback => 0,
            U8 => cnt * mem::size_of::<u8>(),
            U16 => cnt * mem::size_of::<u16>(),
            F32 => cnt * mem::size_of::<f32>(),
        }
    }

    pub(crate) fn wire_type(self) -> u8 {
        use MessageType::*;
        match self {
            Callback => 0,
            U8 => 5,
            U16 => 8,
            F32 => 11,
        }
    }

    pub(crate) fn from_wire(wire: u8) -> Option<Self> {
        use MessageType::*;
        Some(match wire {
            0 => Callback,
            5 => U8,
            8 => U16,
            11 => F32,
            _ => return None,
        })
    }
}
