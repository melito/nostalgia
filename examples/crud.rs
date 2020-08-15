use nostalgia::{record::Record, Storage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum Party {
    Democrat,
    Republican,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mayor {
    id: usize,
    name: std::string::String,
    party: Party,
}

impl Record for Mayor {
    fn key(&self) -> Vec<u8> {
        self.id.to_be_bytes().to_vec()
    }

    fn db_name() -> &'static str {
        "Mayors"
    }
}

fn main() {
    // Mayors of New York City.
    let mayor_infos = vec![
        ("John P. Mitchel", Party::Republican),
        ("John F. Hylan", Party::Democrat),
        ("William T. Collins", Party::Democrat),
        ("Jimmy Walker", Party::Democrat),
        ("Joseph V. McKee", Party::Democrat),
        ("John P. O'Brien", Party::Democrat),
        ("Fiorello H. La Guardia", Party::Republican),
        ("William O'Dwyer", Party::Democrat),
        ("Vincent R. Impellitteri", Party::Democrat),
        ("Robert F. Wagner Jr.", Party::Democrat),
        ("John Lindsay", Party::Republican),
        ("Abraham Beame", Party::Democrat),
        ("Ed Koch", Party::Democrat),
        ("David Dinkins", Party::Democrat),
        ("Rudy Guiliani", Party::Republican),
        ("Michael Bloomberg", Party::Republican),
        ("Warren Wilhelm Jr.", Party::Democrat),
    ];

    // Make an Vec of Person records out of the info we have
    let mayors = mayor_infos
        .iter()
        .enumerate()
        .map(|(idx, m)| Mayor {
            id: idx,
            name: String::from(m.0),
            party: m.1.clone(),
        })
        .collect();

    // Ok now open storage
    let mut storage = Storage::new(std::env::temp_dir()).expect("Could not open/create storage");

    // Use the save batch function to save the entire array
    storage
        .save_batch(mayors)
        .expect("Could not save records in batch");

    let query = storage.query::<Mayor>().unwrap();
    let republicans = query.filter(|i| i.party == Party::Republican);

    for republican in republicans {
        println!("{:?}", republican);
    }
}
