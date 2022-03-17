use core::{fmt, str};

// TODO - consider using [u8] instead of str, can still do cmp with b"foo"
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct MessageId<'a>(&'a str);

impl<'a> MessageId<'a> {
    /// Maximun size in bytes
    pub const MAX_SIZE: usize = crate::wire::packet::Packet::<&[u8]>::MAX_MSG_ID_SIZE;

    pub const INTERNAL_LIB_VER: Self = MessageId("o");
    pub const INTERNAL_BOARD_ID: Self = MessageId("i");
    pub const INTERNAL_HEARTBEAT: Self = MessageId("h");

    pub const fn new(id: &'static str) -> Option<Self> {
        if id.is_empty() || id.len() > Self::MAX_SIZE || (id.len() == 1 && id.as_bytes()[0] == 0) {
            None
        } else {
            Some(Self(id))
        }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub const fn as_str(&self) -> &str {
        self.0
    }

    pub fn from_utf8(bytes: &'a [u8]) -> Option<Self> {
        str::from_utf8(bytes).map(MessageId).ok()
    }
}

impl<'a> From<MessageId<'a>> for &'a str {
    fn from(id: MessageId<'a>) -> Self {
        id.0
    }
}

impl<'a> fmt::Display for MessageId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
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
        assert_eq!(MessageId::INTERNAL_LIB_VER.as_bytes(), b"o");
        assert_eq!(MessageId::INTERNAL_BOARD_ID.as_bytes(), b"i");
        assert_eq!(MessageId::INTERNAL_HEARTBEAT.as_bytes(), b"h");
    }
}
