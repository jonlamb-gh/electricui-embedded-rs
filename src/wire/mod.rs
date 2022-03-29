pub use framing::Framing;
pub use packet::Packet;

pub mod framing;
pub mod packet;

pub(crate) type Field = ::core::ops::Range<usize>;
pub(crate) type Rest = ::core::ops::RangeFrom<usize>;
