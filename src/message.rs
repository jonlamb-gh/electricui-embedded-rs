use core::convert::TryFrom;
use core::{fmt, mem, str};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct MessageId<'a>(&'a [u8]);

impl<'a> MessageId<'a> {
    /// Maximun size in bytes
    pub const MAX_SIZE: usize = crate::wire::packet::Packet::<&[u8]>::MAX_MSG_ID_SIZE;

    pub const INTERNAL_LIB_VER: Self = MessageId(b"o");
    pub const INTERNAL_BOARD_ID: Self = MessageId(b"i");
    pub const INTERNAL_HEARTBEAT: Self = MessageId(b"h");

    /// Announce writable ID's
    pub const INTERNAL_AM: Self = MessageId(b"t");
    /// Delimit writable ID
    pub const INTERNAL_AM_LIST: Self = MessageId(b"u");
    /// End of writable ID's
    pub const INTERNAL_AM_END: Self = MessageId(b"v");
    /// Send writable variables
    pub const INTERNAL_AV: Self = MessageId(b"w");

    pub const BOARD_NAME: Self = MessageId(b"name");

    pub const fn new(id: &'a [u8]) -> Option<Self> {
        if id.is_empty() || id.len() > Self::MAX_SIZE || (id.len() == 1 && id[0] == 0) {
            None
        } else {
            Some(Self(id))
        }
    }

    /// # Safety
    /// Must follow the rules
    pub const unsafe fn new_unchecked(id: &'a [u8]) -> Self {
        Self(id)
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0
    }

    pub fn as_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.0)
    }

    pub fn from_utf8(s: &'a str) -> Self {
        Self(s.as_bytes())
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<MessageId<'a>> for &'a [u8] {
    fn from(id: MessageId<'a>) -> Self {
        id.0
    }
}

impl<'a> TryFrom<&'a MessageId<'a>> for &'a str {
    type Error = str::Utf8Error;

    fn try_from(id: &'a MessageId<'a>) -> Result<Self, Self::Error> {
        id.as_str()
    }
}

// MessageId == [u8]
impl<'a> PartialEq<[u8]> for MessageId<'a> {
    fn eq(&self, other: &[u8]) -> bool {
        self.0 == other
    }
}

// MessageId == &[u8; N]
impl<'a, const N: usize> PartialEq<&[u8; N]> for MessageId<'a> {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.0 == *other
    }
}

// [u8] == MessageId
impl<'a> PartialEq<MessageId<'a>> for [u8] {
    fn eq(&self, other: &MessageId) -> bool {
        self == other.0
    }
}

// &[u8; N] == MessageId
impl<'a, const N: usize> PartialEq<MessageId<'a>> for &[u8; N] {
    fn eq(&self, other: &MessageId) -> bool {
        *self == other.0
    }
}

impl<'a> fmt::Display for MessageId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = self.as_str() {
            f.write_str(s)
        } else {
            write!(f, "{:X?}", self.0)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MessageType {
    Callback,
    Custom,
    OffsetMetadata,
    Byte,
    Char,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
    Unknown(u8),
}

impl MessageType {
    /// Returns the wire size for this MessageType variant.
    /// Only applicable to data carrying types.
    pub fn wire_size_hint(self) -> usize {
        use MessageType::*;
        match self {
            Callback | Custom | Unknown(_) => 0, // Up to the user
            OffsetMetadata => 0,                 // TODO - add offset support
            Byte | Char | I8 | U8 => mem::size_of::<u8>(),
            I16 | U16 => mem::size_of::<u16>(),
            I32 | U32 => mem::size_of::<u32>(),
            F32 => mem::size_of::<f32>(),
            F64 => mem::size_of::<f64>(),
        }
    }

    /// Returns the wire size for an array of this MessageType variant.
    /// Only applicable to data carrying types.
    pub fn array_wire_size_hint(self, num_elements: usize) -> usize {
        num_elements * self.wire_size_hint()
    }

    /// Returns the number of elements for this MessageType
    /// and data payload size.
    /// Only applicable to data carrying types.
    pub fn array_wire_length_hint(self, data_size: usize) -> usize {
        let wire_size = self.wire_size_hint();
        if wire_size == 0 {
            0
        } else {
            data_size / wire_size
        }
    }
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        use MessageType::*;
        match value {
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
            _ => Unknown(value),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        use MessageType::*;
        match value {
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
            Unknown(typ) => typ,
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
pub(crate) mod propt {
    use super::*;
    use proptest::{
        collection, num,
        prelude::*,
        prop_oneof,
        std_facade::{vec, Vec},
    };

    pub fn gen_message_type() -> impl Strategy<Value = MessageType> {
        prop_oneof![
            Just(MessageType::Callback),
            Just(MessageType::Custom),
            Just(MessageType::OffsetMetadata),
            Just(MessageType::Byte),
            Just(MessageType::Char),
            Just(MessageType::I8),
            Just(MessageType::U8),
            Just(MessageType::I16),
            Just(MessageType::U16),
            Just(MessageType::I32),
            Just(MessageType::U32),
            Just(MessageType::F32),
            Just(MessageType::F64),
            gen_unknown_msg_typ(),
        ]
    }

    prop_compose! {
        fn gen_unknown_msg_typ()(value in 13_u8..=0x0F_u8) -> MessageType {
            MessageType::Unknown(value)
        }
    }

    prop_compose! {
        pub fn gen_msg_id_bytes()(bytes in collection::vec(num::u8::ANY, 1..=MessageId::MAX_SIZE)) -> Vec<u8> {
            bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use propt::*;
    use proptest::prelude::*;

    #[test]
    fn internal_ids() {
        assert_eq!(MessageId::INTERNAL_LIB_VER, b"o");
        assert_eq!(MessageId::INTERNAL_BOARD_ID, b"i");
        assert_eq!(MessageId::INTERNAL_HEARTBEAT, b"h");
        assert_eq!(MessageId::INTERNAL_AM, b"t");
        assert_eq!(MessageId::INTERNAL_AM_LIST, b"u");
        assert_eq!(MessageId::INTERNAL_AM_END, b"v");
        assert_eq!(MessageId::INTERNAL_AV, b"w");

        assert_eq!(MessageId::new(b"name"), Some(MessageId::BOARD_NAME));
    }

    #[test]
    fn invalid_ids() {
        assert_eq!(MessageId::new(&[]), None);
        assert_eq!(MessageId::new(&[0]), None);
        let id_bytes: [u8; 16] = [1; 16];
        assert_eq!(id_bytes.len(), MessageId::MAX_SIZE + 1);
        assert_eq!(MessageId::new(&id_bytes), None);
    }

    proptest! {
        #[test]
        fn round_trip_message_type(v_in in gen_message_type()) {
            let wire = u8::from(v_in);
            let v_out = MessageType::from(wire);
            assert_eq!(v_in, v_out);
        }

        #[test]
        fn round_trip_message_id(id_bytes in gen_msg_id_bytes()) {
            if id_bytes.len() == 1 && id_bytes[0] == 0 {
                assert_eq!(MessageId::new(id_bytes.as_ref()), None);
            } else {
                let len = id_bytes.len();
                let s = str::from_utf8(id_bytes.as_ref());
                let id = MessageId::new(id_bytes.as_ref()).unwrap();
                assert_eq!(len, id.len());
                assert_eq!(s, id.as_str());
            }
        }

        #[test]
        fn round_trip_size_helpers(typ in gen_message_type(), num_elements in 1_usize..64_usize) {
            use MessageType::*;
            let wire_size = typ.array_wire_size_hint(num_elements);
            let cnt = typ.array_wire_length_hint(wire_size);
            match typ {
                Callback | Custom | Unknown(_) | OffsetMetadata => {
                    assert_eq!(wire_size, 0);
                    assert_eq!(cnt, 0);
                }
                _ => assert_eq!(cnt, num_elements, "{typ} {wire_size}"),
            }
        }
    }
}
