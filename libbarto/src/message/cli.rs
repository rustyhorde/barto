// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use anyhow::{Error, Result};
use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{AllowedEnumVariants, DecodeError, EncodeError},
};

/// Messages from barto-cli to bartos
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BartoCli {
    /// An info request from barto-cli
    Info {
        /// Output the information in JSON format
        json: bool,
    },
    /// A request from barto-cli to check for recent updates to a bartoc client
    Updates {
        /// The name of the bartoc client to check for recent updates
        name: String,
        /// The update kind we are requesting
        kind: UpdateKind,
    },
    /// A request from barto-cli to clean up old database entries
    Cleanup,
    /// The currently connected clients
    Clients,
    /// A query to run on bartos
    Query {
        /// The query to run on bartos
        query: String,
    },
    /// A request to list the output for a given command
    List {
        /// The name of the bartoc client to check for recent updates
        name: String,
        /// The name of the command to list the output for
        cmd_name: String,
    },
    /// A request to list the jobs that failed
    Failed,
}

impl<Context> Decode<Context> for BartoCli {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let json: bool = Decode::decode(decoder)?;
                Ok(BartoCli::Info { json })
            }
            1 => {
                let name: String = Decode::decode(decoder)?;
                let kind: UpdateKind = Decode::decode(decoder)?;
                Ok(BartoCli::Updates { name, kind })
            }
            2 => Ok(BartoCli::Cleanup),
            3 => Ok(BartoCli::Clients),
            4 => {
                let query: String = Decode::decode(decoder)?;
                Ok(BartoCli::Query { query })
            }
            5 => {
                let name: String = Decode::decode(decoder)?;
                let cmd_name: String = Decode::decode(decoder)?;
                Ok(BartoCli::List { name, cmd_name })
            }
            6 => Ok(BartoCli::Failed),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartoCli",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 6 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BartoCli {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let json: bool = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartoCli::Info { json })
            }
            1 => {
                let name: String = BorrowDecode::borrow_decode(decoder)?;
                let kind: UpdateKind = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartoCli::Updates { name, kind })
            }
            2 => Ok(BartoCli::Cleanup),
            3 => Ok(BartoCli::Clients),
            4 => {
                let query: String = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartoCli::Query { query })
            }
            5 => {
                let name: String = BorrowDecode::borrow_decode(decoder)?;
                let cmd_name: String = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartoCli::List { name, cmd_name })
            }
            6 => Ok(BartoCli::Failed),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartoCli",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 6 },
                found: variant,
            }),
        }
    }
}

impl Encode for BartoCli {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            BartoCli::Info { json } => {
                0u32.encode(encoder)?;
                json.encode(encoder)
            }
            BartoCli::Updates { name, kind } => {
                1u32.encode(encoder)?;
                name.encode(encoder)?;
                kind.encode(encoder)
            }
            BartoCli::Cleanup => 2u32.encode(encoder),
            BartoCli::Clients => 3u32.encode(encoder),
            BartoCli::Query { query } => {
                4u32.encode(encoder)?;
                query.encode(encoder)
            }
            BartoCli::List { name, cmd_name } => {
                5u32.encode(encoder)?;
                name.encode(encoder)?;
                cmd_name.encode(encoder)
            }
            BartoCli::Failed => 6u32.encode(encoder),
        }
    }
}

/// The update kind we are requesting
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateKind {
    /// A garuda-update message
    Garuda,
    /// An Archlinux pacman update message
    Pacman,
    /// A `CachyOS` update message
    Cachyos,
    /// An apt update message
    Apt,
}

impl<Context> Decode<Context> for UpdateKind {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => Ok(UpdateKind::Garuda),
            1 => Ok(UpdateKind::Pacman),
            2 => Ok(UpdateKind::Cachyos),
            3 => Ok(UpdateKind::Apt),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "UpdateKind",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 3 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for UpdateKind {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => Ok(UpdateKind::Garuda),
            1 => Ok(UpdateKind::Pacman),
            2 => Ok(UpdateKind::Cachyos),
            3 => Ok(UpdateKind::Apt),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "UpdateKind",
                allowed: &AllowedEnumVariants::Range { min: 0, max: 3 },
                found: variant,
            }),
        }
    }
}

impl Encode for UpdateKind {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        let variant: u32 = match self {
            UpdateKind::Garuda => 0,
            UpdateKind::Pacman => 1,
            UpdateKind::Cachyos => 2,
            UpdateKind::Apt => 3,
        };
        variant.encode(encoder)
    }
}

impl TryFrom<&str> for UpdateKind {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "garuda" => Ok(UpdateKind::Garuda),
            "pacman" => Ok(UpdateKind::Pacman),
            "cachyos" => Ok(UpdateKind::Cachyos),
            "apt" => Ok(UpdateKind::Apt),
            _ => Err(crate::Error::InvalidUpdateKind {
                kind: value.to_string(),
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use bincode::{borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec};

    use super::{BartoCli, UpdateKind};

    #[test]
    fn test_update_kind_try_from() {
        assert_eq!(UpdateKind::try_from("garuda").unwrap(), UpdateKind::Garuda);
        assert_eq!(UpdateKind::try_from("PACMAN").unwrap(), UpdateKind::Pacman);
        assert_eq!(
            UpdateKind::try_from("Cachyos").unwrap(),
            UpdateKind::Cachyos
        );
        assert_eq!(UpdateKind::try_from("apt").unwrap(), UpdateKind::Apt);
        assert!(UpdateKind::try_from("unknown").is_err());
    }

    #[test]
    fn test_update_kind_encode_decode() {
        let kinds = [
            UpdateKind::Garuda,
            UpdateKind::Pacman,
            UpdateKind::Cachyos,
            UpdateKind::Apt,
        ];
        for kind in &kinds {
            let encoded = encode_to_vec(kind, standard()).unwrap();
            let (decoded, _): (UpdateKind, usize) =
                decode_from_slice(&encoded, standard()).unwrap();
            let (borrowed, _): (UpdateKind, usize) =
                borrow_decode_from_slice(&encoded, standard()).unwrap();
            assert_eq!(*kind, decoded);
            assert_eq!(*kind, borrowed);
        }
    }

    #[test]
    fn test_bartocli_encode_decode() {
        let commands = [
            BartoCli::Info { json: true },
            BartoCli::Updates {
                name: "test".to_string(),
                kind: UpdateKind::Pacman,
            },
            BartoCli::Cleanup,
            BartoCli::Clients,
            BartoCli::Query {
                query: "SELECT * FROM test".to_string(),
            },
            BartoCli::List {
                name: "test".to_string(),
                cmd_name: "list".to_string(),
            },
            BartoCli::Failed,
        ];

        for command in &commands {
            let encoded = encode_to_vec(command, standard()).unwrap();
            let (decoded, _): (BartoCli, usize) = decode_from_slice(&encoded, standard()).unwrap();
            let (borrowed, _): (BartoCli, usize) =
                borrow_decode_from_slice(&encoded, standard()).unwrap();
            assert_eq!(*command, decoded);
            assert_eq!(*command, borrowed);
        }
    }

    #[test]
    fn test_update_kind_encode_invalid_variant() {
        let invalid_variant: u32 = 99;
        let encoded = encode_to_vec(invalid_variant, standard()).unwrap();
        let result: Result<(UpdateKind, _)> =
            decode_from_slice(&encoded, standard()).map_err(Into::into);
        let result_borrow: Result<(UpdateKind, _)> =
            borrow_decode_from_slice(&encoded, standard()).map_err(Into::into);
        assert!(result.is_err());
        assert!(result_borrow.is_err());
    }

    #[test]
    fn test_barto_cli_encode_invalid_variant() {
        use super::BartoCli;

        let invalid_variant: u32 = 99;
        let encoded = encode_to_vec(invalid_variant, standard()).unwrap();
        let result: Result<(BartoCli, _)> =
            decode_from_slice(&encoded, standard()).map_err(Into::into);
        let result_borrow: Result<(BartoCli, _)> =
            borrow_decode_from_slice(&encoded, standard()).map_err(Into::into);
        assert!(result.is_err());
        assert!(result_borrow.is_err());
    }
}
