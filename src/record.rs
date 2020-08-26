use std::convert::{From, Into};
use std::marker::Sized;
use std::string::String;

use serde::{de::DeserializeOwned, Serialize};

/// When a type conforms to this trait it allows it to be stored and retrieved from the database
pub trait Record: Serialize + DeserializeOwned + Sized {
    type Key: Into<Vec<u8>>;

    /// Used to determine the key to use to associate with the object in the database
    fn key(&self) -> Self::Key;

    /// The database name to save a record in.  Defaults to 'default'
    fn db_name() -> &'static str {
        "default"
    }

    /// Serializes the record to binary
    fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes a record from binary
    fn from_binary(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

// A struct to wrap any sized type that could be used as a key
pub struct Key<T: Sized>(T);

// Implement From for any Sized type and wrap it in a Key struct
impl<T> From<T> for Key<T> {
    fn from(input: T) -> Self {
        Key::<T> { 0: input }
    }
}

impl Into<Key<String>> for Key<&str> {
    fn into(self) -> Key<String> {
        Key::<String> {
            0: self.0.to_string(),
        }
    }
}

impl Into<Vec<u8>> for Key<u32> {
    fn into(self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

impl Into<Vec<u8>> for Key<u64> {
    fn into(self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

impl Into<Vec<u8>> for Key<String> {
    fn into(self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Into<Vec<u8>> for Key<&str> {
    fn into(self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Thing {
        id: u32,
    }

    impl Record for Thing {
        type Key = Key<u32>;

        fn key(&self) -> Self::Key {
            Key::from(self.id)
        }
    }

    #[derive(Serialize, Deserialize)]
    struct OtherThing {
        id: std::string::String,
    }

    impl Record for OtherThing {
        type Key = std::string::String;

        fn key(&self) -> Self::Key {
            //Key::from(self.id.clone())
            self.id.clone()
        }
    }

    #[derive(Serialize, Deserialize)]
    struct AnotherThing {
        id: u32,
    }

    impl Record for AnotherThing {
        type Key = Key<u32>;

        fn key(&self) -> Self::Key {
            Key::from(self.id)
        }
    }

    // A dummy function that ensures things compile
    fn get<T: Record>(_key: T::Key) {}

    // Another dummy function to use to see if we can transform inputs and what that looks like
    fn get2<T: Record, K: Into<T::Key>>(_key: K) -> Option<T> {
        None
    }

    #[test]
    fn test_that_are_key_type_is_useful() {
        let a = Key::<u8> { 0: 0 };
        assert_eq!(0, a.0);

        let b = Key::from(8);
        assert_eq!(8, b.0);

        let c = Key::from("Yo");
        assert_eq!("Yo", c.0);

        let d = Key::from(String::from("LOL"));
        assert_eq!("LOL".to_string(), d.0);

        let _e = get::<Thing>(Key::from(8));
        let _f = get::<OtherThing>(String::from("LOL"));
        let g = Key::from([0, 1, 2, 3]);
        assert_eq!([0, 1, 2, 3], g.0);

        let _h: Option<Thing> = get2(8);
        let _i: Option<OtherThing> = get2("Hello".to_string());
        let _j: Option<OtherThing> = get2("Hi");
    }
}
