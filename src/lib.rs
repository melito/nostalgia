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
    /// Create a new database
    /// Accepts a str as an argument.  If directory does not exist it will create it
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

    fn save<T: Storable>(&mut self, record: &T) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;
        let bytes = Storable::to_binary(record).expect("Could not serialize");
        tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;

        tx.commit()
    }

    fn batch_save<T: Storable>(&mut self, records: Vec<T>) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;

        let mut tx = self.env.begin_rw_txn()?;

        for record in records {
            let bytes = Storable::to_binary(&record).expect("Could not serialize");
            tx.put(db, &record.key(), &bytes, lmdb::WriteFlags::empty())?;
        }

        tx.commit()
    }

    fn get<T: Storable + Retrievable>(&mut self, key: &[u8]) -> Result<Option<T>, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;
        let cursor = txn.open_ro_cursor(db)?;
        let result = cursor.get(Some(key), None, 15)?;

        match T::from_binary(result.1) {
            Ok(record) => Ok(Some(record)),
            Err(_) => Ok(None),
        }
    }

    fn query<'txn, T: Storable + Retrievable>(&mut self) -> Result<RoQuery<T>, lmdb::Error> {
        let db = self.db(T::db_name())?;

        let txn = self.env.begin_ro_txn()?;

        Ok(RoQuery {
            phantom: std::marker::PhantomData::<T>,
            db: db,
            txn: txn,
            iter: None,
        })
    }

    fn truncate<T: Storable>(&mut self) -> Result<(), lmdb::Error> {
        let db = self.db(T::db_name())?;
        let mut txn = self.env.begin_rw_txn()?;
        txn.clear_db(db)?;
        txn.commit()?;
        Ok(())
    }

    fn drop<T: Storable>(&mut self) -> Result<(), lmdb::Error> {
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

struct RoQuery<'txn, T> {
    phantom: std::marker::PhantomData<T>,
    db: lmdb::Database,
    txn: lmdb::RoTransaction<'txn>,
    iter: Option<lmdb::Iter<'txn>>,
}

impl<'txn, T: 'txn + Retrievable> Iterator for RoQuery<'txn, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.iter {
            let mut cursor = self.txn.open_ro_cursor(self.db).unwrap();
            self.iter = Some(cursor.iter());
        }

        if let Some(iter) = &mut self.iter {
            if let Some(record) = iter.next() {
                return match T::from_binary(record.1) {
                    Ok(record) => Some(record),
                    Err(_) => None,
                };
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
        #[dummy(faker = "1..1000")]
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

    fn clear_db(storage: &mut Storage) {
        match storage.truncate::<Person>() {
            Ok(_) => assert_eq!(0, 0),
            Err(_) => assert_ne!(0, 0, "Could not truncate Person db"),
        }
    }

    #[test]
    fn test_that_we_can_init_the_db() {
        let mut storage = Storage::new("/tmp/db").expect("Could not open db storage");
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
        let mut storage = Storage::new("/tmp/db").expect("Could not open db storage");
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
        let records_to_create: u32 = 10;
        let mut records: Vec<Person> = vec![];
        for idx in 0..records_to_create {
            records.push(Person {
                id: idx,
                name: Name().fake(),
            });
        }

        let mut storage = Storage::new("/tmp/db").expect("Could not open db storage");
        clear_db(&mut storage);

        let _ = storage.batch_save(records).expect("Could not save records");
        let person_iterator = storage.query::<Person>().unwrap();

        let mut cnt = 0;
        for _ in person_iterator {
            cnt += 1;
        }

        assert_eq!(records_to_create, cnt);
    }
}
