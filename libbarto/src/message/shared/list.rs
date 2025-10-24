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

use crate::OffsetDataTimeWrapper;

/// The output of a `List` request, containing the names of all registered bartoc clients
#[derive(Builder, Clone, CopyGetters, Debug, Decode, Encode, Eq, Getters, PartialEq)]
pub struct ListOutput {
    /// The timestamp of when the output was generated
    #[getset(get = "pub")]
    timestamp: Option<OffsetDataTimeWrapper>,
    /// The data returned from the command
    #[getset(get = "pub")]
    data: Option<String>,
    /// The exit code of the command
    #[getset(get_copy = "pub")]
    exit_code: u8,
    /// Whether the command was successful
    #[getset(get_copy = "pub")]
    success: i8,
}

#[cfg(test)]
mod test {
    use super::ListOutput;

    use anyhow::Result;
    use bincode::{config::standard, decode_from_slice, encode_to_vec};

    #[test]
    fn test_initialize_encode_decode() -> Result<()> {
        let list_output = ListOutput::builder()
            .data("client1\nclient2\n".to_string())
            .exit_code(0)
            .success(1)
            .build();

        let encoded = encode_to_vec(list_output.clone(), standard())?;
        let (decoded_wrapper, _): (ListOutput, _) = decode_from_slice(&encoded, standard())?;

        assert_eq!(list_output, decoded_wrapper);
        assert_eq!(list_output.data(), decoded_wrapper.data());
        assert_eq!(list_output.exit_code(), decoded_wrapper.exit_code());
        assert_eq!(list_output.success(), decoded_wrapper.success());
        assert!(!format!("{list_output:?}").is_empty());
        Ok(())
    }
}
