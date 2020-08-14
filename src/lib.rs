use failure::Error;
use lmdb;
use lmdb::{Cursor, Database, Environment, Transaction};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::PathBuf;

mod query;
pub mod record;

use query::RoQuery;
use record::Record;

/// Storage provides a simple interface for interacting with databases
pub struct Storage {
    env: Environment,
    path: PathBuf,
    dbs: HashMap<&'static str, lmdb::Database>,
}

impl Storage {
    /// Creates or Opens a storage directory for managing databases.
    ///
    /// LMDB storage expects path to be a directory.
    ///
    /// If the path does not exist it will be created.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the database should be created / opened
    ///
    /// # Examples
    ///
    /// ```
    /// use flashback::Storage;
    ///
    /// fn main() -> Result<(), failure::Error> {
    ///     // Into trait allows for str argument
    ///     let a = Storage::new("/tmp/db")?;
    ///
    ///     // Also allows for a std::string::String
    ///     let b = Storage::new(String::from("/tmp/db2"))?;
    ///
    ///     // PathBuf's also work
    ///     let c = Storage::new(std::env::temp_dir())?;
    ///
    ///     Ok(())
    /// }
    ///
    /// ```
    ///
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Storage, Error> {
        let mut builder = lmdb::Environment::new();
        builder.set_max_dbs(2048);
        builder.set_map_size(256 * 1024 * 1024);

        let p = &path.into();
        create_dir_all(p)?;
        let env = builder.open(p).unwrap();

        Ok(Storage {
            env: env,
            path: p.to_path_buf(),
            dbs: HashMap::new(),
        })
    }

    fn db(&mut self, db_name: &'static str) -> Result<Database, lmdb::Error> {
        match self.dbs.get(db_name) {
            Some(db) => Ok(*db),
            None => {
                let db = self
                    .env
                    .create_db(Some(db_name), lmdb::DatabaseFlags::empty())?;
                self.dbs.insert(db_name, db);
                Ok(db)
            }
        }
    }

    /// Serializes and Saves a record in one of the databases contained in storage.
    ///
    /// Input should implement the Record trait.  The database the record is saved to and the key
    /// used is configured using that trait.
    ///
    /// # Arguments
    /// * `record` - A type that implements the Record trait.
    ///
    /// # Examples
    /// ```
    /// use flashback::{Storage, record::Record};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Place {
    ///   id: usize,
    ///   name: std::string::String
    /// }
    ///
    /// impl Record for Place {
    ///    fn key(&self) -> Vec<u8> {
    ///        self.id.to_be_bytes().to_vec()
    ///    }
    ///
    ///    fn db_name() -> &'static str {
    ///        "Place"
    ///    }
    /// }
    ///
    /// fn main() -> Result<(), failure::Error> {
    ///     let mut storage = Storage::new("/tmp/db")?;
    ///     let place = Place { id: 1, name: "Vienna".to_string() };
    ///     storage.save(&place)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn save<T: Record>(&mut self, record: &T) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;
        let bytes = T::to_binary(record).expect("Could not serialize");
        tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;

        tx.commit()
    }

    /// Saves a group of records to the internal type's database
    pub fn batch_save<T: Record>(&mut self, records: Vec<T>) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;

        for record in records {
            let bytes = T::to_binary(&record).expect("Could not serialize");
            tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;
        }

        tx.commit()
    }

    /// Retrieves a record from the database
    pub fn get<T: Record>(&mut self, key: &[u8]) -> Result<Option<T>, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;
        let cursor = txn.open_ro_cursor(db)?;
        let result = cursor.get(Some(key), None, 15)?;

        match T::from_binary(result.1) {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None),
        }
    }

    /// Performs a query on the database and returns an iterator for accessing results
    pub fn query<'txn, T: Record>(&mut self) -> Result<RoQuery<T>, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;

        Ok(RoQuery {
            phantom: std::marker::PhantomData::<T>,
            db: db,
            txn: txn,
            iter: None,
        })
    }

    /// Removes all records in the corresponding type's database
    pub fn truncate<T: Record>(&mut self) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;
        let mut txn = self.env.begin_rw_txn()?;
        txn.clear_db(db)?;
        txn.commit()?;
        Ok(())
    }

    /// Completely removes the database for a specific type
    pub fn drop<T: Record>(&mut self) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;
        let mut txn = self.env.begin_rw_txn()?;
        unsafe {
            txn.drop_db(db)?;
        }
        txn.commit()?;

        self.dbs.remove(T::db_name());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::name::en::Name;
    use fake::{Dummy, Fake, Faker};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, Dummy, PartialEq)]
    struct Person {
        #[dummy(faker = "1..1000")]
        id: u32,

        #[dummy(faker = "Name()")]
        name: String,
    }

    impl Record for Person {
        fn key(&self) -> Vec<u8> {
            self.id.to_be_bytes().to_vec()
        }

        fn db_name() -> &'static str {
            "Person"
        }
    }

    fn clear_db(storage: &mut Storage) {
        match storage.truncate::<Person>() {
            Ok(_) => assert_eq!(0, 0),
            Err(_) => assert_ne!(0, 0, "Could not truncate Person db"),
        }
    }

    #[test]
    fn test_that_we_keep_track_of_db_references() {
        let mut storage = Storage::new(std::env::temp_dir()).expect("Could not open db storage");
        assert_eq!(0, storage.dbs.len());

        let p: Person = Faker.fake();
        storage.save(&p).expect("Could not save record");
        assert_eq!(1, storage.dbs.len());

        match storage.drop::<Person>() {
            Ok(_) => assert_eq!(0, storage.dbs.len()),
            Err(_) => assert_ne!(0, 0, "Could not drop database"),
        }
    }

    #[test]
    fn test_that_we_can_insert_and_get_records_with_a_storage_object() {
        let mut storage = Storage::new(std::env::temp_dir()).expect("Could not open db storage");
        clear_db(&mut storage);

        let person: Person = Faker.fake();

        assert_eq!("Person", Person::db_name());

        let _ = storage.save(&person).expect("Could not save record");
        let p = storage.get::<Person>(&person.key());

        match p {
            Ok(Some(pn)) => assert_eq!(pn, person),
            Ok(None) => assert_ne!(0, 0, "Didn't get a result back"),
            Err(_) => assert_ne!(0, 0, "Got an error"),
        };
    }

    #[test]
    fn test_that_we_can_batch_insert_records_and_then_interate() {
        let records_to_create: u32 = 10000;
        let mut records: Vec<Person> = vec![];
        for idx in 0..records_to_create {
            records.push(Person {
                id: idx,
                name: Name().fake(),
            });
        }

        let mut storage = Storage::new(std::env::temp_dir()).expect("Could not open db storage");
        clear_db(&mut storage);

        let _ = storage.batch_save(records).expect("Could not save records");
        let person_iterator = storage.query::<Person>().unwrap();

        let mut cnt = 0;
        for _ in person_iterator {
            cnt += 1;
        }

        assert_eq!(records_to_create, cnt);

        let pi = storage.query::<Person>().unwrap();
        let x = pi.filter(|i| i.name == "Jerry");

        for y in x {
            println!("{:?}", y);
        }
    }
}
