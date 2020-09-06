use std::convert::Into;
use std::marker::Sized;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Key;
    use crate::Storage;
    use serde::{Deserialize, Serialize};

    #[derive(Storable, Serialize, Deserialize)]
    #[key = "id"]
    struct Thing {
        id: u32,
        body: String,
    }

    #[test]
    fn test_that_we_can_use_the_custom_derive_macro() {
        let mut storage = Storage::new("/tmp/db").expect("Couldn't open database");

        let thing = Thing {
            id: 1,
            body: "Whoa, thing.".to_string(),
        };

        storage.save(&thing).expect("Could not save record");
    }
}
