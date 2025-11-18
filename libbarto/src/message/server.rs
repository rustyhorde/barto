// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::collections::{BTreeMap, HashMap};

use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use vergen_pretty::PrettyExt;

use crate::{
    FailedOutput, Initialize, UpdateKind, UuidWrapper,
    message::shared::{list::ListOutput, sys::ClientData},
};

/// A message from bartos to bartoc
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BartosToBartoc {
    /// Initialize bartoc with the given schedules
    Initialize(Initialize),
}

impl<Context> Decode<Context> for BartosToBartoc {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let initialize_data: Initialize = Decode::decode(decoder)?;
                Ok(BartosToBartoc::Initialize(initialize_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartocWs",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 2 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BartosToBartoc {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let initialize_data: Initialize = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoc::Initialize(initialize_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartocWs",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 2 },
                found: variant,
            }),
        }
    }
}

impl Encode for BartosToBartoc {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            BartosToBartoc::Initialize(initialize_data) => {
                0u32.encode(encoder)?;
                initialize_data.encode(encoder)
            }
        }
    }
}

/// A message from bartos to barto-cli
#[derive(Clone, Debug, PartialEq)]
pub enum BartosToBartoCli {
    /// Information about the bartos server
    Info(PrettyExt),
    /// Information about the bartos server in JSON format
    InfoJson(String),
    /// Updates about a named bartoc client
    Updates(UpdateKind),
    /// Result of a cleanup operation
    Cleanup((u64, u64)),
    /// Current connected clients
    Clients(HashMap<UuidWrapper, ClientData>),
    /// Result of a query operation
    Query(BTreeMap<usize, BTreeMap<String, String>>),
    /// Result of a list operation
    List(Vec<ListOutput>),
    /// Result of a failed command operation request
    Failed(Vec<FailedOutput>),
    /// Result of a list commands operation
    ListCommands(Vec<String>),
    /// Result of a command data by name operation
    Cmd(BTreeMap<String, Vec<ListOutput>>),
}

impl<Context> Decode<Context> for BartosToBartoCli {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => {
                let info_data: PrettyExt = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Info(info_data))
            }
            1 => {
                let info_json_data: String = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::InfoJson(info_json_data))
            }
            2 => {
                let updates_data: UpdateKind = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Updates(updates_data))
            }
            3 => {
                let cleanup_data: (u64, u64) = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Cleanup(cleanup_data))
            }
            4 => {
                let clients_data: HashMap<UuidWrapper, ClientData> = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Clients(clients_data))
            }
            5 => {
                let query_data: BTreeMap<usize, BTreeMap<String, String>> =
                    Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Query(query_data))
            }
            6 => {
                let list_data: Vec<ListOutput> = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::List(list_data))
            }
            7 => {
                let failed_data: Vec<FailedOutput> = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Failed(failed_data))
            }
            8 => {
                let list_commands_data: Vec<String> = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::ListCommands(list_commands_data))
            }
            9 => {
                let cmd_data: BTreeMap<String, Vec<ListOutput>> = Decode::decode(decoder)?;
                Ok(BartosToBartoCli::Cmd(cmd_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartosToBartoCli",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 9 },
                found: variant,
            }),
        }
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BartosToBartoCli {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let variant: u32 = BorrowDecode::borrow_decode(decoder)?;
        match variant {
            0 => {
                let info_data: PrettyExt = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Info(info_data))
            }
            1 => {
                let info_json_data: String = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::InfoJson(info_json_data))
            }
            2 => {
                let updates_data: UpdateKind = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Updates(updates_data))
            }
            3 => {
                let cleanup_data: (u64, u64) = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Cleanup(cleanup_data))
            }
            4 => {
                let clients_data: HashMap<UuidWrapper, ClientData> =
                    BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Clients(clients_data))
            }
            5 => {
                let query_data: BTreeMap<usize, BTreeMap<String, String>> =
                    BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Query(query_data))
            }
            6 => {
                let list_data: Vec<ListOutput> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::List(list_data))
            }
            7 => {
                let failed_data: Vec<FailedOutput> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Failed(failed_data))
            }
            8 => {
                let list_commands_data: Vec<String> = BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::ListCommands(list_commands_data))
            }
            9 => {
                let cmd_data: BTreeMap<String, Vec<ListOutput>> =
                    BorrowDecode::borrow_decode(decoder)?;
                Ok(BartosToBartoCli::Cmd(cmd_data))
            }
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "BartosToBartoCli",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 9 },
                found: variant,
            }),
        }
    }
}

impl Encode for BartosToBartoCli {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            BartosToBartoCli::Info(info_data) => {
                0u32.encode(encoder)?;
                info_data.encode(encoder)
            }
            BartosToBartoCli::InfoJson(info_json_data) => {
                1u32.encode(encoder)?;
                info_json_data.encode(encoder)
            }
            BartosToBartoCli::Updates(updates_data) => {
                2u32.encode(encoder)?;
                updates_data.encode(encoder)
            }
            BartosToBartoCli::Cleanup(cleanup_data) => {
                3u32.encode(encoder)?;
                cleanup_data.encode(encoder)
            }
            BartosToBartoCli::Clients(clients_data) => {
                4u32.encode(encoder)?;
                clients_data.encode(encoder)
            }
            BartosToBartoCli::Query(query_data) => {
                5u32.encode(encoder)?;
                query_data.encode(encoder)
            }
            BartosToBartoCli::List(list_data) => {
                6u32.encode(encoder)?;
                list_data.encode(encoder)
            }
            BartosToBartoCli::Failed(failed_data) => {
                7u32.encode(encoder)?;
                failed_data.encode(encoder)
            }
            BartosToBartoCli::ListCommands(list_commands_data) => {
                8u32.encode(encoder)?;
                list_commands_data.encode(encoder)
            }
            BartosToBartoCli::Cmd(cmd_data) => {
                9u32.encode(encoder)?;
                cmd_data.encode(encoder)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::{BartosToBartoCli, BartosToBartoc};

    use crate::FailedOutput;
    use crate::Initialize;
    use crate::UpdateKind;
    use crate::utils::Mock as _;
    use bincode::{borrow_decode_from_slice, decode_from_slice};
    use bincode::{config::standard, encode_to_vec};
    use vergen_pretty::{Pretty, PrettyExt, vergen_pretty_env};

    #[test]
    fn test_bartos_to_bartoc_initialize_encode_decode() {
        let init = Initialize::mock();
        let msg = BartosToBartoc::Initialize(init.clone());

        let encoded = encode_to_vec(&msg, standard()).unwrap();
        let (decoded, _): (BartosToBartoc, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoc, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(msg, decoded);
        assert_eq!(msg, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_info_roundtrip() {
        let pretty = Pretty::builder().env(vergen_pretty_env!()).build();
        let pretty_ext = PrettyExt::from(pretty);
        let original = BartosToBartoCli::Info(pretty_ext);

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_cleanup_roundtrip() {
        let original = BartosToBartoCli::Cleanup((42, 100));

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_clients_roundtrip() {
        let original = BartosToBartoCli::Clients(HashMap::new());

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_query_roundtrip() {
        let original = BartosToBartoCli::Query(BTreeMap::new());

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_list_roundtrip() {
        let original = BartosToBartoCli::List(Vec::new());

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_failed_roundtrip() {
        let original = BartosToBartoCli::Failed(vec![FailedOutput::mock()]);

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_list_commands_roundtrip() {
        let original =
            BartosToBartoCli::ListCommands(vec!["command1".to_string(), "command2".to_string()]);

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_cmd_roundtrip() {
        let original = BartosToBartoCli::Cmd(BTreeMap::new());

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartoc_invalid_variant() {
        // Manually create an encoded message with invalid variant (e.g., 99)
        let encoded = encode_to_vec(99u32, standard()).unwrap();

        // Try to decode as BartosToBartoc - should fail
        let result: Result<(BartosToBartoc, usize), _> = decode_from_slice(&encoded, standard());
        assert!(result.is_err());

        let result_borrow: Result<(BartosToBartoc, usize), _> =
            borrow_decode_from_slice(&encoded, standard());
        assert!(result_borrow.is_err());
    }

    #[test]
    fn test_bartos_to_bartocli_invalid_variant() {
        // Manually create an encoded message with invalid variant (e.g., 99)
        let encoded = encode_to_vec(99u32, standard()).unwrap();

        // Try to decode as BartosToBartoCli - should fail
        let result: Result<(BartosToBartoCli, usize), _> = decode_from_slice(&encoded, standard());
        assert!(result.is_err());

        let result_borrow: Result<(BartosToBartoCli, usize), _> =
            borrow_decode_from_slice(&encoded, standard());
        assert!(result_borrow.is_err());
    }

    #[test]
    fn test_bartos_to_bartocli_info_json_roundtrip() {
        let json_data = r#"{"version":"1.0.0","build":"test"}"#.to_string();
        let original = BartosToBartoCli::InfoJson(json_data);

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }

    #[test]
    fn test_bartos_to_bartocli_updates_roundtrip() {
        let update = UpdateKind::mock();
        let original = BartosToBartoCli::Updates(update);

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let (decoded, _): (BartosToBartoCli, usize) =
            decode_from_slice(&encoded, standard()).unwrap();
        let (borrowed_decoded, _): (BartosToBartoCli, usize) =
            borrow_decode_from_slice(&encoded, standard()).unwrap();

        assert_eq!(original, decoded);
        assert_eq!(original, borrowed_decoded);
    }
}
