//!## Summary
//! A library that provides syntactic sugar for various file based database systems.
//! Currently provide support for lmdb (Lightning Mapped Database)
//! Planning on adding support for leveldb and RocksDB
//!
//! Using this library allows you to persist and retrieve annotated structs from a
//! database engine using a simple interface.
//!
//! You can query the results using native rust methods like map & filter.
//!
//! ## Goals
//!
//! To provide a useful interface that allows users to quickly take advantage of
//! powerful database systems.  Users should not have to adjust their code to
//! conform to rules required to use some of those database systems.

#[allow(unused_imports)]
#[macro_use]
extern crate nostalgia_derive;

mod key;
mod query;
mod record;
mod storage;

pub use key::Key;
use query::RoQuery;
pub use record::Record;
pub use storage::{Storage, StorageError};
