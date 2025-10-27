// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{AllowedEnumVariants, DecodeError, EncodeError},
};

use crate::{BartocInfo, Data};

/// A supported websocket message from bartoc to bartos
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BartocWs {
    /// A close message from bartoc
    Close(Option<(u16, String)>),
    /// A ping message from bartoc
    Ping(Vec<u8>),
    /// A pong message from bartos
    Pong(Vec<u8>),
}

impl<Context> Decode<Context> for BartocWs {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let close_data: Option<(u16, String)> = Decode::decode(decoder)?;
                Ok(BartocWs::Close(close_data))
            }
            1 => {
                let ping_data: Vec<u8> = Decode::decode(decoder)?;
                Ok(BartocWs::Ping(ping_data))
            }
            2 => {
                let pong_data: Vec<u8> = Decode::decode(decoder)?;
                Ok(BartocWs::Pong(pong_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartocWs",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 2 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BartocWs {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let close_data: Option<(u16, String)> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartocWs::Close(close_data))
            }
            1 => {
                let ping_data: Vec<u8> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartocWs::Ping(ping_data))
            }
            2 => {
                let pong_data: Vec<u8> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartocWs::Pong(pong_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartocWs",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 2 },
                found: variant,
            }),
        }
    }
}

impl Encode for BartocWs {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            BartocWs::Close(close_data) => {
                0u32.encode(encoder)?;
                close_data.encode(encoder)
            }
            BartocWs::Ping(ping_data) => {
                1u32.encode(encoder)?;
                ping_data.encode(encoder)
            }
            BartocWs::Pong(pong_data) => {
                2u32.encode(encoder)?;
                pong_data.encode(encoder)
            }
        }
    }
}

/// A websocket binary message from bartoc to bartos
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bartoc {
    /// A close message from bartoc
    Record(Data),
    /// barto client info
    ClientInfo(BartocInfo),
}

impl<Context> Decode<Context> for Bartoc {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let data: Data = Decode::decode(decoder)?;
                Ok(Bartoc::Record(data))
            }
            1 => {
                let client_info: BartocInfo = Decode::decode(decoder)?;
                Ok(Bartoc::ClientInfo(client_info))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "Bartoc",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 1 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Bartoc {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let data: Data = BorrowDecode::borrow_decode(decoder)?;
                Ok(Bartoc::Record(data))
            }
            1 => {
                let client_info: BartocInfo = BorrowDecode::borrow_decode(decoder)?;
                Ok(Bartoc::ClientInfo(client_info))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "Bartoc",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 1 },
                found: variant,
            }),
        }
    }
}

impl Encode for Bartoc {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            Bartoc::Record(data) => {
                0u32.encode(encoder)?;
                data.encode(encoder)
            }
            Bartoc::ClientInfo(client_info) => {
                1u32.encode(encoder)?;
                client_info.encode(encoder)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bartoc, BartocWs};

    use crate::{BartocInfo, Data, Output, utils::Mock as _};
    use bincode::{borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec};

    #[test]
    fn test_bartoc_ws_encode_decode() {
        let original = BartocWs::Close(None);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartocWs, usize) = decode_from_slice(&encoded, standard()).unwrap();
        let (borrow_decoded, _): (BartocWs, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
    }

    #[test]
    fn test_bartoc_ws_ping_encode_decode() {
        let ping_data = vec![1, 2, 3, 4, 5];
        let original = BartocWs::Ping(ping_data.clone());
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartocWs, usize) = decode_from_slice(&encoded, standard()).unwrap();
        let (borrow_decoded, _): (BartocWs, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
    }

    #[test]
    fn test_bartoc_ws_pong_encode_decode() {
        let pong_data = vec![6, 7, 8, 9, 10];
        let original = BartocWs::Pong(pong_data.clone());
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartocWs, usize) = decode_from_slice(&encoded, standard()).unwrap();
        let (borrow_decoded, _): (BartocWs, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
    }

    #[test]
    fn test_bartoc_ws_bad_variant_decode() {
        // Encode a bad variant (3) manually
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&3u32.to_le_bytes()); // Invalid variant

        let result: Result<(BartocWs, usize), _> = decode_from_slice(&encoded, standard());
        assert!(result.is_err());

        let borrow_result: Result<(BartocWs, usize), _> =
            borrow_decode_from_slice(&encoded, standard());
        assert!(borrow_result.is_err());
    }

    #[test]
    fn test_bartoc_client_info_encode_decode() {
        let client_info = BartocInfo::mock();
        let original = Bartoc::ClientInfo(client_info);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (Bartoc, usize) = decode_from_slice(&encoded, standard()).unwrap();
        let (borrow_decoded, _): (Bartoc, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
    }

    #[test]
    fn test_bartoc_record_encode_decode() {
        let output = Output::mock();
        let original = Bartoc::Record(Data::Output(output));
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (Bartoc, usize) = decode_from_slice(&encoded, standard()).unwrap();
        let (borrow_decoded, _): (Bartoc, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrow_decoded);
    }

    #[test]
    fn test_bartoc_bad_variant_decode() {
        // Encode a bad variant (2) manually
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&2u32.to_le_bytes()); // Invalid variant

        let result: Result<(Bartoc, usize), _> = decode_from_slice(&encoded, standard());
        assert!(result.is_err());

        let borrow_result: Result<(Bartoc, usize), _> =
            borrow_decode_from_slice(&encoded, standard());
        assert!(borrow_result.is_err());
    }
}
