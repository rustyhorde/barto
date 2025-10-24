// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use bincode::{Decode, Encode};
use bon::Builder;
use getset::{CopyGetters, Getters};

use crate::{Schedules, UuidWrapper};

/// An initialization message from bartos to a named bartoc client.
#[derive(Builder, Clone, CopyGetters, Debug, Decode, Encode, Eq, Getters, PartialEq)]
pub struct Initialize {
    /// The unique identifier for the bartoc client
    #[get_copy = "pub"]
    id: UuidWrapper,
    /// The schedules to initialize the bartoc client with
    #[get = "pub"]
    schedules: Schedules,
}

#[cfg(test)]
mod test {
    use super::Initialize;

    use anyhow::Result;
    use bincode::{config::standard, decode_from_slice, encode_to_vec};
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
        let (decoded_wrapper, _): (Initialize, _) = decode_from_slice(&encoded, standard())?;

        assert_eq!(initialize, decoded_wrapper);
        assert_eq!(initialize.id(), decoded_wrapper.id());
        assert_eq!(initialize.schedules(), decoded_wrapper.schedules());
        assert!(!format!("{initialize:?}").is_empty());
        Ok(())
    }
}
