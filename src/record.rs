/// When a type conforms to this trait it allows it to be stored and retrieved from the database
pub trait Record: serde::Serialize + serde::de::DeserializeOwned + std::marker::Sized {
    type Key: std::convert::Into<Vec<u8>>;

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
