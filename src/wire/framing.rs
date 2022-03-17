//! A framing wrapper around https://crates.io/crates/corncobs

use err_derive::Error;

#[derive(Debug, Copy, Clone, Error)]
pub enum Error {
    #[error(display = "{}", _0)]
    Cobs(#[source] corncobs::CobsError),
}

pub struct Framing {}

impl Framing {
    pub const ZERO: u8 = corncobs::ZERO;

    pub const fn max_encoded_len(raw_len: usize) -> usize {
        corncobs::max_encoded_len(raw_len)
    }

    pub fn decode_buf(bytes: &[u8], output: &mut [u8]) -> Result<usize, Error> {
        let b = corncobs::decode_buf(bytes, output)?;
        Ok(b)
    }

    pub fn decode_in_place(bytes: &mut [u8]) -> Result<usize, Error> {
        let b = corncobs::decode_in_place(bytes)?;
        Ok(b)
    }

    pub fn encode_buf(bytes: &[u8], output: &mut [u8]) -> usize {
        corncobs::encode_buf(bytes, output)
    }

    pub fn encode_iter(bytes: &[u8]) -> impl Iterator<Item = u8> + '_ {
        corncobs::encode_iter(bytes)
    }
}
