use failure::Error;
use lmdb;
use lmdb::{Cursor, Database, Environment, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::string::String;

trait Storable: serde::Serialize + std::marker::Sized {
    fn key(&self) -> Vec<u8>;

    fn db_name() -> &'static str {
        "default"
    }

    fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
}

trait Retrievable: serde::de::DeserializeOwned {
    fn from_binary(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

#[derive(Debug)]
pub struct Storage {
    env: Environment,
    path: String,
    dbs: HashMap<&'static str, lmdb::Database>,
}

impl Storage {
    fn new<P: Into<PathBuf> + Copy>(path: P) -> Result<Storage, Error> {
        let mut builder = lmdb::Environment::new();
        builder.set_max_dbs(2048);
        builder.set_map_size(256 * 1024 * 1024);

        let p = path.into();
        create_dir_all(p)?;
        let env = builder.open(&path.into()).unwrap();
        Ok(Storage {
            env: env,
            path: path.into().to_str().unwrap().to_string(),
            dbs: HashMap::new(),
        })
    }

    fn db(&self, db_name: &'static str) -> Result<Database, lmdb::Error> {
        match self.dbs.get(db_name) {
            Some(db) => Ok(*db),
            None => self
                .env
                .create_db(Some(db_name), lmdb::DatabaseFlags::empty()),
        }
    }

    fn save<T: Storable>(&self, record: &T) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;
        let bytes = Storable::to_binary(record).expect("Could not serialize");
        tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;

        tx.commit()
    }

    fn batch_save<T: Storable>(&self, records: Vec<T>) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;

        for record in records {
            let bytes = Storable::to_binary(&record).expect("Could not serialize");
            tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;
        }

        tx.commit()
    }

    fn get<T: Storable + Retrievable>(&self, key: &[u8]) -> Result<Option<T>, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;
        let cursor = txn.open_ro_cursor(db)?;
        let result = cursor.get(Some(key), None, 15)?;

        match T::from_binary(result.1) {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None),
        }
    }

    fn query<'txn, T: Storable + Retrievable>(&self) -> Result<RoQuery, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;

        Ok(RoQuery {
            db: db,
            txn: txn,
            iter: None,
        })
    }
}

struct RoQuery<'txn> {
    db: lmdb::Database,
    txn: lmdb::RoTransaction<'txn>,
    iter: Option<lmdb::Iter<'txn>>,
}

impl<'txn> Iterator for RoQuery<'txn> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.iter {
            let mut cursor = self.txn.open_ro_cursor(self.db).unwrap();
            self.iter = Some(cursor.iter());
        }

        if let Some(iter) = &mut self.iter {
            if let Some(record) = iter.next() {
                println!("{:?}", record);
                return Some(String::new());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::name::en::Name;
    use fake::{Dummy, Fake, Faker};

    #[derive(Debug, Serialize, Deserialize, Dummy, PartialEq)]
    struct Person {
        #[dummy(faker = "1000..2000")]
        id: u32,

        #[dummy(faker = "Name()")]
        name: String,
    }

    impl Storable for Person {
        fn key(&self) -> Vec<u8> {
            self.id.to_be_bytes().to_vec()
        }

        fn db_name() -> &'static str {
            "Person"
        }
    }

    impl Retrievable for Person {}

    #[test]
    fn test_that_we_can_init_the_db() {
        let _ = Storage::new("/tmp/db").expect("Could not open db storage");
    }

    #[test]
    fn test_that_we_can_insert_and_get_records_with_a_storage_object() {
        let storage = Storage::new("/tmp/db").expect("Could not open db storage");

        let person: Person = Faker.fake();

        assert_eq!("Person", Person::db_name());

        let _ = storage.save(&person).expect("Could not save record");
        let p = storage.get::<Person>(&person.key());

        match p {
            Ok(Some(pn)) => assert_eq!(pn, person),
            Ok(None) => assert_ne!(0, 0, "Didn't get a result back"),
            Err(_) => assert_ne!(0, 0, "Got an error"),
        }
    }

    #[test]
    fn test_that_we_can_batch_insert_records_and_then_interate() {
        let mut records: Vec<Person> = vec![];
        for _ in 0..10000 {
            records.push(Faker.fake());
        }

        let storage = Storage::new("/tmp/db").expect("Could not open db storage");
        let _ = storage.batch_save(records).expect("Could not save records");
        let person_iterator = storage.query::<Person>().unwrap();

        for person in person_iterator {
            println!("{:?}", person);
        }
    }
}
