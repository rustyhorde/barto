// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::fmt::{self, Display, Formatter};

use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use bon::Builder;
use getset::{Getters, Setters};
use sysinfo::System;

#[cfg(test)]
use crate::utils::Mock;

/// bartoc client system information
#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq)]
pub struct BartocInfo {
    /// The name of the bartoc client
    #[get = "pub"]
    #[builder(default = System::name().unwrap_or_default())]
    name: String,
    /// The hostname of the bartoc client
    #[get = "pub"]
    #[builder(default = System::host_name().unwrap_or_default())]
    hostname: String,
    /// The operating system version of the bartoc client
    #[get = "pub"]
    #[builder(default = System::os_version().unwrap_or_default())]
    os_version: String,
    /// The kernel version of the bartoc client
    #[get = "pub"]
    #[builder(default = System::kernel_version().unwrap_or_default())]
    kernel_version: String,
}

#[cfg(test)]
impl Mock for BartocInfo {
    fn mock() -> Self {
        Self::builder()
            .name("mock_client".to_string())
            .hostname("mock_host".to_string())
            .os_version("1.0.0".to_string())
            .kernel_version("5.10.0".to_string())
            .build()
    }
}

impl<Context> Decode<Context> for BartocInfo {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            name: Decode::decode(decoder)?,
            hostname: Decode::decode(decoder)?,
            os_version: Decode::decode(decoder)?,
            kernel_version: Decode::decode(decoder)?,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for BartocInfo {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            name: BorrowDecode::borrow_decode(decoder)?,
            hostname: BorrowDecode::borrow_decode(decoder)?,
            os_version: BorrowDecode::borrow_decode(decoder)?,
            kernel_version: BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl Encode for BartocInfo {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.name, encoder)?;
        Encode::encode(&self.hostname, encoder)?;
        Encode::encode(&self.os_version, encoder)?;
        Encode::encode(&self.kernel_version, encoder)?;
        Ok(())
    }
}

impl Display for BartocInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.name, self.os_version, self.kernel_version
        )
    }
}

/// bartoc client data
#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq, Setters)]
pub struct ClientData {
    /// bartoc client name
    #[getset(get = "pub")]
    #[builder(default)]
    name: String,
    /// bartoc client ip
    #[getset(get = "pub")]
    #[builder(default)]
    ip: String,
    /// bartoc client system information
    #[getset(get = "pub", set = "pub")]
    bartoc_info: Option<BartocInfo>,
}

impl<Context> Decode<Context> for ClientData {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            name: Decode::decode(decoder)?,
            ip: Decode::decode(decoder)?,
            bartoc_info: Decode::decode(decoder)?,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for ClientData {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            name: BorrowDecode::borrow_decode(decoder)?,
            ip: BorrowDecode::borrow_decode(decoder)?,
            bartoc_info: BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl Encode for ClientData {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.name, encoder)?;
        Encode::encode(&self.ip, encoder)?;
        Encode::encode(&self.bartoc_info, encoder)?;
        Ok(())
    }
}

impl Display for ClientData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(bartoc_info) = &self.bartoc_info {
            write!(f, "{bartoc_info}")
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{BartocInfo, ClientData};

    use anyhow::Result;
    use bincode::{borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec};

    #[test]
    fn test_bartoc_info_encode_decode() -> Result<()> {
        let bartoc_info = BartocInfo::builder()
            .name("test_client".to_string())
            .hostname("test_host".to_string())
            .os_version("1.0.0".to_string())
            .kernel_version("5.10.0".to_string())
            .build();
        let encoded = encode_to_vec(bartoc_info.clone(), standard())?;
        let (decoded, _): (BartocInfo, _) = decode_from_slice(&encoded, standard())?;
        let (borrow_decoded, _): (BartocInfo, _) = borrow_decode_from_slice(&encoded, standard())?;
        assert_eq!(bartoc_info, decoded);
        assert_eq!(bartoc_info, borrow_decoded);
        Ok(())
    }

    #[test]
    fn test_client_data_encode_decode() -> Result<()> {
        let bartoc_info = BartocInfo::builder()
            .name("test_client".to_string())
            .hostname("test_host".to_string())
            .os_version("1.0.0".to_string())
            .kernel_version("5.10.0".to_string())
            .build();
        let client_data = ClientData::builder()
            .name("client1".to_string())
            .ip("192.168.1.1".to_string())
            .bartoc_info(bartoc_info)
            .build();
        let encoded = encode_to_vec(client_data.clone(), standard())?;
        let (decoded, _): (ClientData, _) = decode_from_slice(&encoded, standard())?;
        let (borrow_decoded, _): (ClientData, _) = borrow_decode_from_slice(&encoded, standard())?;
        assert_eq!(client_data, decoded);
        assert_eq!(client_data, borrow_decoded);
        Ok(())
    }

    #[test]
    fn test_bartoc_info_display() {
        let bartoc_info = BartocInfo::builder()
            .name("test_client".to_string())
            .hostname("test_host".to_string())
            .os_version("1.0.0".to_string())
            .kernel_version("5.10.0".to_string())
            .build();
        let formatted = format!("{bartoc_info}");
        assert_eq!(formatted, "test_client 1.0.0 5.10.0");
    }

    #[test]
    fn test_client_data_display() {
        let bartoc_info = BartocInfo::builder()
            .name("test_client".to_string())
            .hostname("test_host".to_string())
            .os_version("1.0.0".to_string())
            .kernel_version("5.10.0".to_string())
            .build();
        let client_data_with_info = ClientData::builder()
            .name("client1".to_string())
            .ip("192.168.1.1".to_string())
            .bartoc_info(bartoc_info)
            .build();
        let formatted = format!("{client_data_with_info}");
        assert_eq!(formatted, "test_client 1.0.0 5.10.0");
        let client_data_without_info = ClientData::builder()
            .name("client2".to_string())
            .ip("192.168.1.2".to_string())
            .build();
        let formatted = format!("{client_data_without_info}");
        assert_eq!(formatted, "client2");
    }
}
