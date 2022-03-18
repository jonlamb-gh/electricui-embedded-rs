use err_derive::Error;

#[derive(Copy, Clone, Debug, Error)]
pub enum Error {
    #[error(display = "Packet error. {}", _0)]
    Packet(crate::wire::packet::Error),

    #[error(display = "Framing error. {}", _0)]
    Framing(crate::wire::framing::Error),

    #[error(display = "Decoder error. {}", _0)]
    Decoder(crate::decoder::Error),
}
