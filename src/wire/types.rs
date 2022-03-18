use crate::message::MessageType;
use core::{cmp, mem};

impl MessageType {
    pub fn data_wire_size(self, num_elements: usize) -> usize {
        use MessageType::*;
        let cnt = cmp::max(1, num_elements);
        match self {
            Callback => 0,
            Custom => 0,         // Up to the user
            OffsetMetadata => 0, // TODO - add offset support
            Byte | Char | I8 | U8 => cnt * mem::size_of::<u8>(),
            I16 | U16 => cnt * mem::size_of::<u16>(),
            I32 | U32 => cnt * mem::size_of::<u32>(),
            F32 => cnt * mem::size_of::<f32>(),
            F64 => cnt * mem::size_of::<f64>(),
        }
    }

    pub(crate) fn wire_type(self) -> u8 {
        use MessageType::*;
        match self {
            Callback => 0,
            Custom => 1,
            OffsetMetadata => 2,
            Byte => 3,
            Char => 4,
            I8 => 5,
            U8 => 6,
            I16 => 7,
            U16 => 8,
            I32 => 9,
            U32 => 10,
            F32 => 11,
            F64 => 12,
        }
    }

    pub(crate) fn from_wire(wire: u8) -> Option<Self> {
        use MessageType::*;
        Some(match wire {
            0 => Callback,
            1 => Custom,
            2 => OffsetMetadata,
            3 => Byte,
            4 => Char,
            5 => I8,
            6 => U8,
            7 => I16,
            8 => U16,
            9 => I32,
            10 => U32,
            11 => F32,
            12 => F64,
            _ => return None,
        })
    }
}
