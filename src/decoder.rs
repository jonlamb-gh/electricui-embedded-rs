use crate::sealed;
use crate::wire::{packet, Packet};
use err_derive::Error;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum Error {
    #[error(display = "Not enough bytes in the decoder buffer to store the frame")]
    InsufficientBufferSize,

    #[error(display = "Encountered a packet error. {}", _0)]
    PacketError(#[error(source)] packet::Error),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum State {
    FrameOffset,
    HeaderB0,
    HeaderB1,
    HeaderB2,
    MsgId,
    OffsetB0,
    OffsetB1,
    Payload,
    CrcB0,
    CrcB1,
}

#[derive(Debug)]
pub struct Decoder<'buf, const N: usize> {
    state: State,

    frame_offset: u8,
    id_bytes_read: u8,
    data_bytes_read: u16,
    bytes_read: usize,
    valid_pkt_count: usize,
    invalid_pkt_count: usize,

    data_len: u16,
    offset: bool,
    id_len: u8,

    packet_storage: &'buf mut [u8; N],
}

impl<'buf, const N: usize> Decoder<'buf, N> {
    pub fn new(packet_storage: &'buf mut [u8; N]) -> Self {
        sealed::greater_than_eq::<N, { Packet::<&[u8]>::BASE_PACKET_SIZE }>();
        Self {
            state: State::FrameOffset,
            frame_offset: 0,
            id_bytes_read: 0,
            data_bytes_read: 0,
            bytes_read: 0,
            valid_pkt_count: 0,
            invalid_pkt_count: 0,
            data_len: 0,
            offset: false,
            id_len: 0,
            packet_storage,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.state = State::FrameOffset;
        self.frame_offset = 0;
        self.bytes_read = 0;
    }

    pub fn count(&self) -> usize {
        self.valid_pkt_count
    }

    pub fn invalid_count(&self) -> usize {
        self.invalid_pkt_count
    }

    pub fn decode(&mut self, mut byte: u8) -> Result<Option<Packet<&[u8]>>, Error> {
        // COBS framing
        if byte == 0x00 {
            self.reset();
            return Ok(None);
        } else if self.frame_offset > 1 {
            // One byte closer to the next offset
            self.frame_offset -= 1;
        } else {
            // Offset has expired, this inbound byte should be the next data framing byte
            self.frame_offset = byte;
            byte = 0x00;
        }

        match self.state {
            State::FrameOffset => {
                // First byte is the first offset
                self.state = State::HeaderB0;
            }
            State::HeaderB0 => {
                self.feed(byte)?;
                self.data_len = byte as _;
                self.state = State::HeaderB1;
            }
            State::HeaderB1 => {
                self.feed(byte)?;
                self.data_len |= ((byte as u16) << 8) & 0x0300;
                self.offset = ((byte >> 7) & 0x01) != 0;
                self.state = State::HeaderB2;
            }
            State::HeaderB2 => {
                self.feed(byte)?;
                self.id_len = byte & 0x0F;
                self.id_bytes_read = 0;
                self.state = State::MsgId;
            }
            State::MsgId => {
                self.feed(byte)?;
                self.id_bytes_read = self.id_bytes_read.saturating_add(1);
                if self.id_bytes_read >= self.id_len {
                    if self.offset {
                        self.state = State::OffsetB0
                    } else if self.data_len > 0 {
                        self.data_bytes_read = 0;
                        self.state = State::Payload;
                    } else {
                        self.state = State::CrcB0;
                    }
                }
            }
            State::OffsetB0 => {
                // TODO - Add support for split/offset packets
                self.feed(byte)?;
                self.state = State::OffsetB1;
            }
            State::OffsetB1 => {
                // TODO - Add support for split/offset packets
                self.feed(byte)?;
                self.state = State::Payload;
            }
            State::Payload => {
                self.feed(byte)?;
                self.data_bytes_read = self.data_bytes_read.saturating_add(1);
                if self.data_bytes_read >= self.data_len {
                    self.state = State::CrcB0;
                }
            }
            State::CrcB0 => {
                self.feed(byte)?;
                self.state = State::CrcB1;
            }
            State::CrcB1 => {
                self.feed(byte)?;
                let bytes_read = self.bytes_read;
                self.reset();
                match Packet::new(&self.packet_storage[..bytes_read]) {
                    Ok(p) => {
                        self.valid_pkt_count = self.valid_pkt_count.saturating_add(1);
                        return Ok(p.into());
                    }
                    Err(e) => {
                        self.invalid_pkt_count = self.invalid_pkt_count.saturating_add(1);
                        return Err(e.into());
                    }
                }
            }
        }

        Ok(None)
    }

    #[inline]
    fn feed(&mut self, byte: u8) -> Result<(), Error> {
        if self.bytes_read >= self.packet_storage.len() {
            Err(Error::InsufficientBufferSize)
        } else {
            self.packet_storage[self.bytes_read] = byte;
            self.bytes_read = self.bytes_read.saturating_add(1);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // TODO - happy/sad path tests

    static MSG_F32: [u8; 12 + 2] = [
        0x00, 0x0D, // framing
        0x04, 0x2c, 0x03, // header
        0x61, 0x62, 0x63, // msgid
        0x14, 0xAE, 0x29, 0x42, // payload
        0x8B, 0x1D, // crc
    ];

    #[test]
    fn basic_decoding() {
        let mut buffer = [0_u8; 512];
        let mut dec = Decoder::new(&mut buffer);

        for _ in 0..4 {
            for (idx, byte) in MSG_F32.iter().enumerate() {
                let maybe_frame = dec.decode(*byte).unwrap();
                if idx < (MSG_F32.len() - 1) {
                    assert_eq!(maybe_frame.is_some(), false);
                } else {
                    assert_eq!(maybe_frame.is_some(), true);
                }
            }
        }

        assert_eq!(dec.count(), 4);
        assert_eq!(dec.invalid_count(), 0);
    }
}
