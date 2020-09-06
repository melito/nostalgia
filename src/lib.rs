#[macro_use]
extern crate nostalgia_derive;

mod key;
mod query;
mod record;
mod storage;

pub use key::Key;
use query::RoQuery;
pub use record::Record;
pub use storage::{Error, Storage};
