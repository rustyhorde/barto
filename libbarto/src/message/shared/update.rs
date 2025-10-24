// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::cmp::Ordering;

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

/// The update kind
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateKind {
    /// A garuda-update message
    Garuda(Vec<Garuda>),
    /// An Archlinux pacman update message
    Pacman(Pacman),
    /// A `CachyOS` update message
    Cachyos(Pacman),
    /// An other update message
    Other,
}

#[cfg(test)]
impl Mock for UpdateKind {
    fn mock() -> Self {
        UpdateKind::Garuda(vec![
            Garuda::builder()
                .channel("stable")
                .package("firefox")
                .old_version("110.0-1")
                .new_version("111.0-1")
                .size_change("+5.2")
                .download_size("85.3")
                .build(),
        ])
    }
}

impl<Context> Decode<Context> for UpdateKind {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let variant: u32 = Decode::decode(decoder)?;
        match variant {
            0 => Ok(UpdateKind::Garuda(Decode::decode(decoder)?)),
            1 => Ok(UpdateKind::Pacman(Decode::decode(decoder)?)),
            2 => Ok(UpdateKind::Cachyos(Decode::decode(decoder)?)),
            3 => Ok(UpdateKind::Other),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "UpdateKind",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 3 },
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
            0 => Ok(UpdateKind::Garuda(BorrowDecode::borrow_decode(decoder)?)),
            1 => Ok(UpdateKind::Pacman(BorrowDecode::borrow_decode(decoder)?)),
            2 => Ok(UpdateKind::Cachyos(BorrowDecode::borrow_decode(decoder)?)),
            3 => Ok(UpdateKind::Other),
            _ => Err(DecodeError::UnexpectedVariant {
                type_name: "UpdateKind",
                allowed: &bincode::error::AllowedEnumVariants::Range { min: 0, max: 3 },
                found: variant,
            }),
        }
    }
}

impl Encode for UpdateKind {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            UpdateKind::Garuda(v) => {
                0u32.encode(encoder)?;
                v.encode(encoder)
            }
            UpdateKind::Pacman(v) => {
                1u32.encode(encoder)?;
                v.encode(encoder)
            }
            UpdateKind::Cachyos(v) => {
                2u32.encode(encoder)?;
                v.encode(encoder)
            }
            UpdateKind::Other => 3u32.encode(encoder),
        }
    }
}

/// A garuda-update message
#[derive(Builder, Clone, Debug, Eq, Getters, PartialEq)]
pub struct Garuda {
    /// The channel the package belongs to
    #[get = "pub"]
    #[builder(into)]
    channel: String,
    /// The package that was updated
    #[get = "pub"]
    #[builder(into)]
    package: String,
    /// The old version of the package
    #[get = "pub"]
    #[builder(into)]
    old_version: String,
    /// The new version of the package
    #[get = "pub"]
    #[builder(into)]
    new_version: String,
    /// The net change in size (in MiB)
    #[get = "pub"]
    #[builder(into)]
    size_change: String,
    /// The download size (in MiB)
    #[get = "pub"]
    #[builder(into)]
    download_size: String,
}

impl Ord for Garuda {
    fn cmp(&self, other: &Self) -> Ordering {
        self.channel
            .cmp(&other.channel)
            .then(self.package.cmp(&other.package))
    }
}

impl PartialOrd for Garuda {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Context> Decode<Context> for Garuda {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let channel: String = Decode::decode(decoder)?;
        let package: String = Decode::decode(decoder)?;
        let old_version: String = Decode::decode(decoder)?;
        let new_version: String = Decode::decode(decoder)?;
        let size_change: String = Decode::decode(decoder)?;
        let download_size: String = Decode::decode(decoder)?;

        Ok(Garuda {
            channel,
            package,
            old_version,
            new_version,
            size_change,
            download_size,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Garuda {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let channel: String = BorrowDecode::borrow_decode(decoder)?;
        let package: String = BorrowDecode::borrow_decode(decoder)?;
        let old_version: String = BorrowDecode::borrow_decode(decoder)?;
        let new_version: String = BorrowDecode::borrow_decode(decoder)?;
        let size_change: String = BorrowDecode::borrow_decode(decoder)?;
        let download_size: String = BorrowDecode::borrow_decode(decoder)?;

        Ok(Garuda {
            channel,
            package,
            old_version,
            new_version,
            size_change,
            download_size,
        })
    }
}

impl Encode for Garuda {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.channel.encode(encoder)?;
        self.package.encode(encoder)?;
        self.old_version.encode(encoder)?;
        self.new_version.encode(encoder)?;
        self.size_change.encode(encoder)?;
        self.download_size.encode(encoder)
    }
}

/// A garuda-update message
#[derive(Builder, Clone, CopyGetters, Debug, Getters, PartialEq)]
pub struct Pacman {
    /// The package update count
    #[get_copy = "pub"]
    update_count: usize,
    /// The channel the package belongs to
    #[get = "pub"]
    #[builder(into)]
    packages: Vec<String>,
    /// The install size (in MiB)
    #[get_copy = "pub"]
    install_size: f64,
    /// The net size change (in MiB)
    #[get_copy = "pub"]
    net_size: f64,
    /// The download size (in MiB)
    #[get_copy = "pub"]
    download_size: f64,
}

impl<Context> Decode<Context> for Pacman {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let update_count: usize = Decode::decode(decoder)?;
        let packages: Vec<String> = Decode::decode(decoder)?;
        let install_size: f64 = Decode::decode(decoder)?;
        let net_size: f64 = Decode::decode(decoder)?;
        let download_size: f64 = Decode::decode(decoder)?;

        Ok(Pacman {
            update_count,
            packages,
            install_size,
            net_size,
            download_size,
        })
    }
}

impl<'de, Context> BorrowDecode<'de, Context> for Pacman {
    fn borrow_decode<D: BorrowDecoder<'de, Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let update_count: usize = BorrowDecode::borrow_decode(decoder)?;
        let packages: Vec<String> = BorrowDecode::borrow_decode(decoder)?;
        let install_size: f64 = BorrowDecode::borrow_decode(decoder)?;
        let net_size: f64 = BorrowDecode::borrow_decode(decoder)?;
        let download_size: f64 = BorrowDecode::borrow_decode(decoder)?;

        Ok(Pacman {
            update_count,
            packages,
            install_size,
            net_size,
            download_size,
        })
    }
}

impl Encode for Pacman {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.update_count.encode(encoder)?;
        self.packages.encode(encoder)?;
        self.install_size.encode(encoder)?;
        self.net_size.encode(encoder)?;
        self.download_size.encode(encoder)
    }
}

#[cfg(test)]
mod tests {
    use super::{Garuda, Pacman, UpdateKind};
    use bincode::{config::standard, decode_from_slice, encode_to_vec};

    #[test]
    fn test_update_kind_garuda_encode_decode() {
        let garuda_updates = vec![
            Garuda::builder()
                .channel("stable")
                .package("firefox")
                .old_version("110.0-1")
                .new_version("111.0-1")
                .size_change("+5.2")
                .download_size("85.3")
                .build(),
            Garuda::builder()
                .channel("testing")
                .package("kernel")
                .old_version("6.1.0-1")
                .new_version("6.2.0-1")
                .size_change("+12.8")
                .download_size("156.7")
                .build(),
        ];

        let original = UpdateKind::Garuda(garuda_updates);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_pacman_encode_decode() {
        let pacman = Pacman::builder()
            .update_count(42)
            .packages(vec!["firefox".to_string(), "kernel".to_string()])
            .install_size(256.5)
            .net_size(-12.3)
            .download_size(128.7)
            .build();

        let original = UpdateKind::Pacman(pacman);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_cachyos_encode_decode() {
        let cachyos = Pacman::builder()
            .update_count(15)
            .packages(vec!["mesa".to_string(), "nvidia".to_string()])
            .install_size(512.1)
            .net_size(25.4)
            .download_size(89.9)
            .build();

        let original = UpdateKind::Cachyos(cachyos);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_other_encode_decode() {
        let original = UpdateKind::Other;
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_garuda_encode_decode() {
        let original = Garuda::builder()
            .channel("chaotic-aur")
            .package("yay-git")
            .old_version("12.1.2-1")
            .new_version("12.1.3-1")
            .size_change("+0.1")
            .download_size("2.4")
            .build();

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: Garuda = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_pacman_encode_decode() {
        let original = Pacman::builder()
            .update_count(7)
            .packages(vec![
                "linux".to_string(),
                "systemd".to_string(),
                "glibc".to_string(),
            ])
            .install_size(1024.0)
            .net_size(50.5)
            .download_size(256.8)
            .build();

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: Pacman = decode_from_slice(&encoded, standard()).unwrap().0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_garuda_ordering() {
        let garuda1 = Garuda::builder()
            .channel("stable")
            .package("firefox")
            .old_version("110.0-1")
            .new_version("111.0-1")
            .size_change("+5.2")
            .download_size("85.3")
            .build();

        let garuda2 = Garuda::builder()
            .channel("stable")
            .package("chromium")
            .old_version("110.0-1")
            .new_version("111.0-1")
            .size_change("+5.2")
            .download_size("85.3")
            .build();

        let garuda3 = Garuda::builder()
            .channel("testing")
            .package("firefox")
            .old_version("110.0-1")
            .new_version("111.0-1")
            .size_change("+5.2")
            .download_size("85.3")
            .build();

        assert!(garuda2 < garuda1); // chromium < firefox in same channel
        assert!(garuda1 < garuda3); // stable < testing
    }

    #[test]
    fn test_update_kind_garuda_borrow_decode() {
        let garuda_updates = vec![
            Garuda::builder()
                .channel("stable")
                .package("firefox")
                .old_version("110.0-1")
                .new_version("111.0-1")
                .size_change("+5.2")
                .download_size("85.3")
                .build(),
        ];

        let original = UpdateKind::Garuda(garuda_updates);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_pacman_borrow_decode() {
        let pacman = Pacman::builder()
            .update_count(42)
            .packages(vec!["firefox".to_string(), "kernel".to_string()])
            .install_size(256.5)
            .net_size(-12.3)
            .download_size(128.7)
            .build();

        let original = UpdateKind::Pacman(pacman);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_cachyos_borrow_decode() {
        let cachyos = Pacman::builder()
            .update_count(15)
            .packages(vec!["mesa".to_string(), "nvidia".to_string()])
            .install_size(512.1)
            .net_size(25.4)
            .download_size(89.9)
            .build();

        let original = UpdateKind::Cachyos(cachyos);
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_other_borrow_decode() {
        let original = UpdateKind::Other;
        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: UpdateKind = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_garuda_borrow_decode() {
        let original = Garuda::builder()
            .channel("chaotic-aur")
            .package("yay-git")
            .old_version("12.1.2-1")
            .new_version("12.1.3-1")
            .size_change("+0.1")
            .download_size("2.4")
            .build();

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: Garuda = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_pacman_borrow_decode() {
        let original = Pacman::builder()
            .update_count(7)
            .packages(vec![
                "linux".to_string(),
                "systemd".to_string(),
                "glibc".to_string(),
            ])
            .install_size(1024.0)
            .net_size(50.5)
            .download_size(256.8)
            .build();

        let encoded = encode_to_vec(&original, standard()).unwrap();
        let decoded: Pacman = bincode::borrow_decode_from_slice(&encoded, standard())
            .unwrap()
            .0;

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_update_kind_invalid_variant() {
        // Create invalid encoded data with variant 99
        let mut invalid_data = encode_to_vec(99u32, standard()).unwrap();
        invalid_data.extend_from_slice(&encode_to_vec("dummy", standard()).unwrap());

        let result: Result<(UpdateKind, usize), _> = decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_garuda_invalid_decode() {
        // Create invalid encoded data with incomplete fields
        let mut invalid_data = encode_to_vec("valid_channel", standard()).unwrap();
        invalid_data.extend_from_slice(&encode_to_vec("valid_package", standard()).unwrap());
        // Missing remaining fields - should fail to decode

        let result: Result<(Garuda, usize), _> = decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_pacman_invalid_decode() {
        // Create invalid encoded data with incomplete fields
        let mut invalid_data = encode_to_vec(42usize, standard()).unwrap();
        invalid_data
            .extend_from_slice(&encode_to_vec(vec!["firefox".to_string()], standard()).unwrap());
        // Missing remaining fields - should fail to decode

        let result: Result<(Pacman, usize), _> = decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_garuda_malformed_data() {
        // Create completely malformed data
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];

        let result: Result<(Garuda, usize), _> = decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_pacman_malformed_data() {
        // Create completely malformed data
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];

        let result: Result<(Pacman, usize), _> = decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }

    #[test]
    fn test_update_kind_garuda_invalid_variant_borrow_decode() {
        // Create invalid encoded data with variant 99 for borrow decode
        let mut invalid_data = encode_to_vec(99u32, standard()).unwrap();
        invalid_data.extend_from_slice(&encode_to_vec("dummy", standard()).unwrap());

        let result: Result<(UpdateKind, usize), _> =
            bincode::borrow_decode_from_slice(&invalid_data, standard());
        assert!(result.is_err());
    }
}
