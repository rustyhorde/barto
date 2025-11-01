// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{any::type_name, cmp::Ordering, fmt::Debug};

use bincode::{Decode, Encode, config::standard, decode_from_slice, encode_to_vec};
use redb::{Key, TypeName, Value};

/// A generic newtype to handle redb keys and values that implement `bincode::Encode` and `bincode::Decode`
#[derive(Debug)]
pub struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
where
    T: Debug + Encode + Decode<()>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;

    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        decode_from_slice(data, standard()).unwrap().0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        encode_to_vec(value, standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}

impl<T> Key for Bincode<T>
where
    T: Debug + Decode<()> + Encode + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::Bincode;
    use bincode::{Decode, Encode};
    use redb::{Database, ReadableDatabase as _, TableDefinition};
    use tempfile::NamedTempFile;

    #[derive(Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct TestKey {
        id: u32,
    }

    #[derive(Clone, Debug, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct TestValue {
        id: u32,
        name: String,
    }
    static TEST_TABLE: TableDefinition<'_, Bincode<TestKey>, Bincode<TestValue>> =
        TableDefinition::new("test_table");

    #[test]
    fn test_bincode_key_value() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        let key = TestKey { id: 42u32 };
        let value = TestValue {
            id: 1,
            name: "Test".to_string(),
        };
        let db = Database::create(path).unwrap();

        {
            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(TEST_TABLE).unwrap();
                let _old = table.insert(&key, &value).unwrap();
            }
            write_txn.commit().unwrap();
        }
        {
            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(TEST_TABLE).unwrap();

            let key = TestKey { id: 42u32 };
            let retrieved_value = table.get(&key).unwrap().unwrap();

            assert_eq!(
                retrieved_value.value(),
                TestValue {
                    id: 1,
                    name: "Test".to_string(),
                }
            );
        }
    }
}
