use crate::message::{MessageId, MessageType};
use byteorder::{ByteOrder, LittleEndian};
use core::fmt;
use crc::{Algorithm, Crc};
use err_derive::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum Error {
    #[error(display = "Not enough bytes for a valid header")]
    MissingHeader,

    #[error(display = "Not enough bytes for a valid header and checksum")]
    MissingChecksum,

    #[error(display = "Not enough bytes for a valid payload according to the data length")]
    IncompletePayload,

    #[error(display = "Invalid checksum")]
    InvalidChecksum,

    #[error(display = "Invalid message ID length")]
    InvalidMessageIdLength,

    #[error(display = "Invalid message ID")]
    InvalidMessageId,

    #[error(display = "Invalid data length")]
    InvalidDataLength,

    #[error(display = "Unknown message type ({})", _0)]
    UnknownMessageType(u8),
}

#[derive(Debug, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::{Field, Rest};

    // Header field byte indices and ranges
    pub const DATA_LEN: Field = 0..2;
    pub const TYPE: usize = 1;
    pub const INTERNAL: usize = 1;
    pub const OFFSET: usize = 1;
    pub const ID_LEN: usize = 2;
    pub const RESPONSE: usize = 2;
    pub const ACKNUM: usize = 2;

    // Message ID bytes followed by maybe offset and maybe packet payload
    pub const REST: Rest = 3..;
    // Followed by 2 byte checksum
}

impl<T: AsRef<[u8]>> Packet<T> {
    pub const HEADER_SIZE: usize = 3;
    pub const CHECKSUM_SIZE: usize = 2;
    pub const OFFSET_SIZE: usize = 2;
    pub const MAX_PAYLOAD_SIZE: usize = 1024;
    pub const MAX_MSG_ID_SIZE: usize = 15;

    pub const BASE_PACKET_SIZE: usize = Self::HEADER_SIZE + Self::CHECKSUM_SIZE;

    pub const MAX_PACKET_SIZE: usize =
        Self::BASE_PACKET_SIZE + Self::MAX_MSG_ID_SIZE + Self::MAX_PAYLOAD_SIZE;

    pub const CRC16_CCITT_FALSE: Algorithm<u16> = Algorithm {
        poly: 0x1021,
        init: 0xFFFF,
        refin: false,
        refout: false,
        xorout: 0,
        check: 0x29B1,
        residue: 0,
    };

    pub fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    pub fn new(buffer: T) -> Result<Packet<T>, Error> {
        let p = Self::new_unchecked(buffer);
        p.check_len()?;
        p.check_payload_length()?;
        p.check_checksum()?;
        Ok(p)
    }

    pub fn check_len(&self) -> Result<(), Error> {
        let len = self.buffer.as_ref().len();
        if len < field::REST.start {
            Err(Error::MissingHeader)
        } else if len < (field::REST.start + Self::CHECKSUM_SIZE) {
            Err(Error::MissingChecksum)
        } else {
            Ok(())
        }
    }

    /// Checks that the buffer contains enough bytes to read
    /// both the message ID and the payload bytes
    pub fn check_payload_length(&self) -> Result<(), Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let len = self.buffer.as_ref().len();
        if len < Self::buffer_len(id_len, data_len) {
            Err(Error::IncompletePayload)
        } else {
            Ok(())
        }
    }

    pub fn check_checksum(&self) -> Result<(), Error> {
        let provided = self.checksum()?;
        let computed = self.compute_checksum()?;
        if computed != provided {
            Err(Error::InvalidChecksum)
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn wire_size(&self) -> Result<usize, Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        Ok(Self::buffer_len(id_len, data_len))
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the length of a buffer required to hold a message
    /// with a payload length of `n_msg_id_bytes` + `n_payload_bytes`.
    #[inline]
    pub fn buffer_len(n_msg_id_bytes: usize, n_payload_bytes: usize) -> usize {
        Self::BASE_PACKET_SIZE + n_msg_id_bytes + n_payload_bytes
    }

    #[inline]
    pub fn data_length(&self) -> u16 {
        let data = self.buffer.as_ref();
        LittleEndian::read_u16(&data[field::DATA_LEN]) & 0x3FF
    }

    #[inline]
    pub fn typ_raw(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::TYPE] >> 2) & 0x0F
    }

    #[inline]
    pub fn typ(&self) -> Result<MessageType, Error> {
        let typ = self.typ_raw();
        MessageType::from_wire(typ).ok_or(Error::UnknownMessageType(typ))
    }

    #[inline]
    pub fn internal(&self) -> bool {
        let data = self.buffer.as_ref();
        ((data[field::INTERNAL] >> 6) & 0x01) != 0
    }

    #[inline]
    pub fn offset(&self) -> bool {
        let data = self.buffer.as_ref();
        ((data[field::OFFSET] >> 7) & 0x01) != 0
    }

    #[inline]
    pub fn id_length_raw(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::ID_LEN] & 0x0F
    }

    #[inline]
    pub fn id_length(&self) -> Result<usize, Error> {
        let id = self.id_length_raw();
        if id == 0 {
            Err(Error::InvalidMessageIdLength)
        } else {
            Ok(id.into())
        }
    }

    #[inline]
    pub fn response(&self) -> bool {
        let data = self.buffer.as_ref();
        ((data[field::RESPONSE] >> 4) & 0x01) != 0
    }

    #[inline]
    pub fn acknum(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::ACKNUM] >> 5) & 0x07
    }

    #[inline]
    pub fn checksum(&self) -> Result<u16, Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let start = field::REST.start + id_len + data_len;
        let end = start + Self::CHECKSUM_SIZE;
        let data = self.buffer.as_ref();
        debug_assert!(end <= data.len());
        Ok(LittleEndian::read_u16(&data[start..end]))
    }

    #[inline]
    pub fn compute_checksum(&self) -> Result<u16, Error> {
        let crc = Crc::<u16>::new(&Self::CRC16_CCITT_FALSE);
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let end = Self::HEADER_SIZE + id_len + data_len;
        let data = self.buffer.as_ref();
        debug_assert!(end <= data.len());
        Ok(crc.checksum(&data[..end]))
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Packet<&'a T> {
    #[inline]
    pub fn msg_id_raw(&self) -> Result<&'a [u8], Error> {
        let id_len = self.id_length()?;
        let end = field::REST.start + id_len;
        let data = self.buffer.as_ref();
        debug_assert!(end <= data.len());
        Ok(&data[field::REST.start..end])
    }

    #[inline]
    pub fn msg_id(&self) -> Result<MessageId<'a>, Error> {
        let msg_id = self.msg_id_raw()?;
        MessageId::new(msg_id).ok_or(Error::InvalidMessageId)
    }

    #[inline]
    pub fn payload(&self) -> Result<&'a [u8], Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let start = field::REST.start + id_len;
        let end = start + data_len;
        let data = self.buffer.as_ref();
        debug_assert!(end <= data.len());
        Ok(&data[start..end])
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    #[inline]
    pub fn set_data_length(&mut self, value: u16) -> Result<(), Error> {
        if usize::from(value) > Self::MAX_PAYLOAD_SIZE {
            Err(Error::InvalidDataLength)
        } else {
            let data = self.buffer.as_mut();
            LittleEndian::write_u16(&mut data[field::DATA_LEN], value & 0x3FF);
            Ok(())
        }
    }

    #[inline]
    pub fn set_typ(&mut self, value: MessageType) {
        let data = self.buffer.as_mut();
        data[field::TYPE] = (data[field::TYPE] & !0x3C) | (value.wire_type() << 2);
    }

    #[inline]
    pub fn set_internal(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        if value {
            data[field::INTERNAL] |= 1 << 6;
        } else {
            data[field::INTERNAL] &= !(1 << 6);
        }
    }

    #[inline]
    pub fn set_offset(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        if value {
            data[field::OFFSET] |= 1 << 7;
        } else {
            data[field::OFFSET] &= !(1 << 7);
        }
    }

    #[inline]
    pub fn set_id_length(&mut self, value: u8) -> Result<(), Error> {
        if value == 0 || usize::from(value) > Self::MAX_MSG_ID_SIZE {
            Err(Error::InvalidMessageIdLength)
        } else {
            let data = self.buffer.as_mut();
            data[field::ID_LEN] = (data[field::ID_LEN] & !0x0F) | (value & 0x0F);
            Ok(())
        }
    }

    #[inline]
    pub fn set_response(&mut self, value: bool) {
        let data = self.buffer.as_mut();
        if value {
            data[field::RESPONSE] |= 1 << 4;
        } else {
            data[field::RESPONSE] &= !(1 << 4);
        }
    }

    #[inline]
    pub fn set_acknum(&mut self, value: u8) {
        let data = self.buffer.as_mut();
        data[field::ACKNUM] = (data[field::ACKNUM] & !0xE0) | ((value & 0x07) << 5);
    }

    #[inline]
    pub fn msg_id_mut(&mut self) -> Result<&mut [u8], Error> {
        let id_len = self.id_length()?;
        let end = field::REST.start + id_len;
        let data = self.buffer.as_mut();
        debug_assert!(end <= data.len());
        Ok(&mut data[field::REST.start..end])
    }

    #[inline]
    pub fn payload_mut(&mut self) -> Result<&mut [u8], Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let start = field::REST.start + id_len;
        let end = start + data_len;
        let data = self.buffer.as_mut();
        debug_assert!(end <= data.len());
        Ok(&mut data[start..end])
    }

    #[inline]
    pub fn set_checksum(&mut self, value: u16) -> Result<(), Error> {
        let id_len = self.id_length()?;
        let data_len = usize::from(self.data_length());
        let start = field::REST.start + id_len + data_len;
        let end = start + Self::CHECKSUM_SIZE;
        let data = self.buffer.as_mut();
        debug_assert!(end <= data.len());
        LittleEndian::write_u16(&mut data[start..end], value);
        Ok(())
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Packet<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<T: AsRef<[u8]>> fmt::Display for Packet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ DataLen({}), Type({}), Int({}), Offset({}), IdLen({}), Resp({}), Acknum({}) }}",
            self.data_length(),
            self.typ_raw(),
            self.internal() as u8,
            self.offset() as u8,
            self.id_length_raw(),
            self.response() as u8,
            self.acknum()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::framing::Framing;
    use approx::assert_relative_eq;
    use pretty_assertions::assert_eq;

    static MSG_I8: [u8; 9 + 2] = [
        0x0A, // framing
        0x01, 0x14, 0x63, // header
        0x61, 0x62, 0x63, // msgid
        0x2A, // payload
        0xB8, 0xA3, // crc
        0x00, // framing
    ];

    static MSG_F32: [u8; 12 + 2] = [
        0x0D, // framing
        0x04, 0x2c, 0x03, // header
        0x61, 0x62, 0x63, // msgid
        0x14, 0xAE, 0x29, 0x42, // payload
        0x8B, 0x1D, // crc
        0x00, // framing
    ];

    #[test]
    fn construct_i8() {
        let mut bytes = [0xFF; 9];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert!(p.check_len().is_ok());
        p.set_data_length(1).unwrap();
        p.set_typ(MessageType::I8);
        p.set_internal(false);
        p.set_offset(false);
        p.set_id_length(3).unwrap();
        p.set_response(false);
        p.set_acknum(3);
        p.msg_id_mut().unwrap().copy_from_slice(b"abc");
        p.payload_mut().unwrap()[0] = 0x2A;
        p.set_checksum(0xA3B8).unwrap();
        assert!(p.check_payload_length().is_ok());
        assert!(p.check_checksum().is_ok());
        assert_eq!(p.wire_size(), Ok(9));
        assert_eq!(&p.into_inner()[..], &MSG_I8[1..10]);

        let mut enc_bytes = [0xFF; 9 + 2];
        assert!(enc_bytes.len() == Framing::max_encoded_len(9));
        let size = Framing::encode_buf(&bytes[..], &mut enc_bytes[..]);
        assert_eq!(size, 9 + 2);
        assert_eq!(&enc_bytes[..], &MSG_I8[..]);
    }

    #[test]
    fn deconstruct_i8() {
        let mut bytes = [0xFF; 9];
        let size = Framing::decode_buf(&MSG_I8[..], &mut bytes[..]).unwrap();
        assert_eq!(size, bytes.len());

        assert_eq!(Packet::<&[u8]>::buffer_len(3, 1), bytes.len());
        let p = Packet::new(&bytes[..]).unwrap();
        assert_eq!(p.data_length(), 1);
        assert_eq!(p.typ().unwrap(), MessageType::I8);
        assert_eq!(p.internal(), false);
        assert_eq!(p.offset(), false);
        assert_eq!(p.id_length().unwrap(), 3);
        assert_eq!(p.response(), false);
        assert_eq!(p.acknum(), 3);
        assert_eq!(p.msg_id().unwrap(), b"abc");
        assert_eq!(p.payload().unwrap(), &[0x2A]);
        assert_eq!(p.checksum().unwrap(), 0xA3B8);
        assert_eq!(p.compute_checksum().unwrap(), 0xA3B8);
        assert_eq!(p.wire_size(), Ok(9));
    }

    #[test]
    fn construct_f32() {
        let mut bytes = [0xFF; 12];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert!(p.check_len().is_ok());
        p.set_data_length(4).unwrap();
        p.set_typ(MessageType::F32);
        p.set_internal(false);
        p.set_offset(false);
        p.set_id_length(3).unwrap();
        p.set_response(false);
        p.set_acknum(0);
        p.msg_id_mut().unwrap().copy_from_slice(b"abc");
        LittleEndian::write_f32(p.payload_mut().unwrap(), 42.42_f32);
        p.set_checksum(0x1D8B).unwrap();
        assert!(p.check_payload_length().is_ok());
        assert!(p.check_checksum().is_ok());
        assert_eq!(p.wire_size(), Ok(12));
        assert_eq!(&p.into_inner()[..], &MSG_F32[1..13]);

        let mut enc_bytes = [0xFF; 12 + 2];
        assert!(enc_bytes.len() == Framing::max_encoded_len(12));
        let size = Framing::encode_buf(&bytes[..], &mut enc_bytes[..]);
        assert_eq!(size, 12 + 2);
        assert_eq!(&enc_bytes[..], &MSG_F32[..]);
    }

    #[test]
    fn deconstruct_f32() {
        let mut bytes = [0xFF; 12];
        let size = Framing::decode_buf(&MSG_F32[..], &mut bytes[..]).unwrap();
        assert_eq!(size, bytes.len());

        assert_eq!(Packet::<&[u8]>::buffer_len(3, 4), bytes.len());
        let p = Packet::new(&bytes[..]).unwrap();
        assert_eq!(p.data_length(), 4);
        assert_eq!(p.typ().unwrap(), MessageType::F32);
        assert_eq!(p.internal(), false);
        assert_eq!(p.offset(), false);
        assert_eq!(p.id_length().unwrap(), 3);
        assert_eq!(p.response(), false);
        assert_eq!(p.acknum(), 0);
        assert_eq!(p.msg_id().unwrap(), b"abc");
        assert_eq!(p.payload().unwrap(), &[0x14, 0xAE, 0x29, 0x42]);
        assert_relative_eq!(LittleEndian::read_f32(p.payload().unwrap()), 42.42_f32);
        assert_eq!(p.checksum().unwrap(), 0x1D8B);
        assert_eq!(p.compute_checksum().unwrap(), 0x1D8B);
        assert_eq!(p.wire_size(), Ok(12));
    }

    #[test]
    fn buffer_len() {
        assert_eq!(
            Packet::<&[u8]>::buffer_len(1, 0),
            Packet::<&[u8]>::BASE_PACKET_SIZE + 1
        );
        assert_eq!(
            Packet::<&[u8]>::buffer_len(3, 4),
            Packet::<&[u8]>::BASE_PACKET_SIZE + 3 + 4
        );
    }

    #[test]
    fn missing_header() {
        let bytes = [0xFF; 5 - 3];
        assert_eq!(bytes.len(), Packet::<&[u8]>::buffer_len(0, 0) - 3);
        let p = Packet::new(&bytes[..]);
        assert_eq!(p.unwrap_err(), Error::MissingHeader);
    }

    #[test]
    fn missing_checksum() {
        let bytes = [0xFF; 5 - 1];
        assert_eq!(bytes.len(), Packet::<&[u8]>::buffer_len(0, 0) - 1);
        let p = Packet::new(&bytes[..]);
        assert_eq!(p.unwrap_err(), Error::MissingChecksum);
    }

    #[test]
    fn incomplete_payload() {
        let bytes = [0x04, 0x2c, 0x03, 0xFF, 0xFF];
        let p = Packet::new(&bytes[..]);
        assert_eq!(p.unwrap_err(), Error::IncompletePayload);
    }

    #[test]
    fn invalid_checksum() {
        let bytes = [0x01, 0x14, 0x63, 0x61, 0x62, 0x63, 0x2A, 0xB8, 0xA3 + 1];
        let p = Packet::new(&bytes[..]);
        assert_eq!(p.unwrap_err(), Error::InvalidChecksum);
    }

    #[test]
    fn invalid_msg_id_len() {
        let mut bytes = [0x01, 0x14, 0x63, 0x61, 0x62, 0x63, 0x2A, 0xB8, 0xA3];
        let mut p = Packet::new(&mut bytes[..]).unwrap();
        assert_eq!(
            p.set_id_length(0).unwrap_err(),
            Error::InvalidMessageIdLength
        );
        assert_eq!(
            p.set_id_length(Packet::<&[u8]>::MAX_MSG_ID_SIZE as u8 + 1)
                .unwrap_err(),
            Error::InvalidMessageIdLength
        );
        bytes[field::ID_LEN] &= !0x0F; // zero
        let p = Packet::new(&bytes[..]);
        assert_eq!(p.unwrap_err(), Error::InvalidMessageIdLength);
    }

    #[test]
    fn invalid_msg_id() {
        let mut bytes = [0xFF; 7];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert!(p.check_len().is_ok());
        p.set_data_length(0).unwrap();
        p.set_typ(MessageType::Custom);
        p.set_internal(false);
        p.set_offset(false);
        p.set_id_length(1).unwrap();
        p.set_response(false);
        p.set_acknum(0);
        p.msg_id_mut().unwrap().copy_from_slice(&[0]); // zero invalid
        p.set_checksum(p.compute_checksum().unwrap()).unwrap();
        assert!(p.check_payload_length().is_ok());
        assert!(p.check_checksum().is_ok());

        let p = Packet::new(&bytes[..]).unwrap();
        assert_eq!(p.msg_id().unwrap_err(), Error::InvalidMessageId);
    }

    #[test]
    fn invalid_data_len() {
        let mut bytes = [0xFF; 32];
        let mut p = Packet::new_unchecked(&mut bytes[..]);
        assert!(p.check_len().is_ok());
        assert_eq!(
            p.set_data_length(Packet::<&[u8]>::MAX_PAYLOAD_SIZE as u16 + 1)
                .unwrap_err(),
            Error::InvalidDataLength
        );
    }
}
