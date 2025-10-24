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
    error::{DecodeError, EncodeError},
};
use bon::Builder;
use getset::{CopyGetters, Getters};

#[cfg(test)]
use crate::utils::Mock;
use crate::{Schedules, UuidWrapper};

/// An initialization message from bartos to a named bartoc client.
#[derive(Builder, Clone, CopyGetters, Debug, Eq, Getters, PartialEq)]
pub struct Initialize {
    /// The unique identifier for the bartoc client
    #[get_copy = "pub"]
    id: UuidWrapper,
    /// The schedules to initialize the bartoc client with
    #[get = "pub"]
    schedules: Schedules,
}

#[cfg(test)]
impl Mock for Initialize {
    fn mock() -> Self {
        Self::builder()
            .id(UuidWrapper::mock())
            .schedules(Schedules::mock())
            .build()
    }
}

impl<Context> Decode<Context> for Initialize {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        Ok(Self {
            id: Decode::decode(decoder)?,
            schedules: Decode::decode(decoder)?,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Initialize {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            id: BorrowDecode::borrow_decode(decoder)?,
            schedules: BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl Encode for Initialize {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.id, encoder)?;
        Encode::encode(&self.schedules, encoder)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Initialize;

    use anyhow::Result;
    use bincode::{borrow_decode_from_slice, config::standard, decode_from_slice, encode_to_vec};
    use uuid::Uuid;

    use crate::{Schedule, Schedules, UuidWrapper};

    #[test]
    fn test_initialize_encode_decode() -> Result<()> {
        let uuid_wrapper = UuidWrapper(Uuid::new_v4());
        let schedule = Schedule::builder()
            .name("test_schedule".to_string())
            .on_calendar("*,*,* 10:10:R".to_string())
            .cmds(vec!["echo 'Hello, World!'".to_string()])
            .build();
        let schedules = Schedules::builder().schedules(vec![schedule]).build();
        let initialize = Initialize::builder()
            .id(uuid_wrapper)
            .schedules(schedules)
            .build();

        let encoded = encode_to_vec(initialize.clone(), standard())?;
        let (decoded, _): (Initialize, _) = decode_from_slice(&encoded, standard())?;
        let (borrow_decoded, _): (Initialize, _) = borrow_decode_from_slice(&encoded, standard())?;

        assert_eq!(initialize, decoded);
        assert_eq!(initialize, borrow_decoded);
        assert_eq!(initialize.id(), decoded.id());
        assert_eq!(initialize.schedules(), decoded.schedules());
        assert!(!format!("{initialize:?}").is_empty());
        Ok(())
    }
}
