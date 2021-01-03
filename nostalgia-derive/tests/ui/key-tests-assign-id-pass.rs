use nostalgia::{Key, Record};
use nostalgia_derive::Storable;
use serde::{Deserialize, Serialize};

#[derive(Storable, Serialize, Deserialize)]
#[key = "id"]
struct Thing {
    id: u32,
}

fn main() {}
