use crate::Record;
use lmdb::{Cursor, Transaction};

pub struct RoQuery<'txn, T> {
    pub phantom: std::marker::PhantomData<T>,
    pub db: lmdb::Database,
    pub txn: lmdb::RoTransaction<'txn>,
    pub iter: Option<lmdb::Iter<'txn>>,
}

impl<'txn, T: 'txn + Record> RoQuery<'txn, T> {
    pub fn new(db: lmdb::Database, txn: lmdb::RoTransaction<'txn>) -> RoQuery<'txn, T> {
        RoQuery {
            phantom: std::marker::PhantomData::<T>,
            db,
            txn,
            iter: None,
        }
    }
}

impl<'txn, T: 'txn + Record> Iterator for RoQuery<'txn, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.is_none() {
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
