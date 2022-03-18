use core::convert::TryFrom;
use core::{fmt, str};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct MessageId<'a>(&'a [u8]);

impl<'a> MessageId<'a> {
    /// Maximun size in bytes
    pub const MAX_SIZE: usize = crate::wire::packet::Packet::<&[u8]>::MAX_MSG_ID_SIZE;

    pub const INTERNAL_LIB_VER: Self = MessageId(b"o");
    pub const INTERNAL_BOARD_ID: Self = MessageId(b"i");
    pub const INTERNAL_HEARTBEAT: Self = MessageId(b"h");

    pub const fn new(id: &'a [u8]) -> Option<Self> {
        if id.is_empty() || id.len() > Self::MAX_SIZE || (id.len() == 1 && id[0] == 0) {
            None
        } else {
            Some(Self(id))
        }
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
    U8,
    U16,
    F32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn internal_ids() {
        assert_eq!(MessageId::INTERNAL_LIB_VER, b"o");
        assert_eq!(MessageId::INTERNAL_BOARD_ID, b"i");
        assert_eq!(MessageId::INTERNAL_HEARTBEAT, b"h");
    }
}
