use nostalgia::{Key, Record, Storage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum Party {
    Democrat,
    Republican,
    NoAffiliation,
}

#[derive(Serialize, Deserialize, Debug)]
struct Mayor {
    id: u32,
    name: std::string::String,
    party: Party,
}

impl Record for Mayor {
    type Key = Key<u32>;

    fn key(&self) -> Key<u32> {
        Key::from(self.id)
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
            id: idx as u32,
            name: m.0.to_string(),
            party: m.1.clone(),
        })
        .collect();

    // Ok now open storage
    let mut storage = Storage::new(std::env::temp_dir()).expect("Could not open/create storage");

    // Use the save batch function to save the entire array
    storage
        .save_batch(mayors)
        .expect("Could not save records in batch");

    // Query for all republicans.
    // We can use rust's standard filter function to query by record properties
    list_all_mayors(Party::Republican, &mut storage);

    // Query for all democrats.
    // We can use rust's standard filter function to query by record properties
    list_all_mayors(Party::Democrat, &mut storage);

    // During his second term, Michael Bloomberg switched from being formally recognized as a
    // republican to no party affiliation.  Let's update his record to reflect that.
    let mut bloomberg = storage
        .find::<Mayor>(&|r| r.name == "Michael Bloomberg")
        .expect("Could not execute find")
        .expect("Could not find mayor");

    println!("\n\n================================================");
    println!(
        "{} started his first term as a {:?}",
        bloomberg.name, bloomberg.party
    );
    println!("Changing affiliation to: {:?}", Party::NoAffiliation);
    bloomberg.party = Party::NoAffiliation;

    storage.save(&bloomberg).expect("Could not update record");

    list_all_mayors(Party::NoAffiliation, &mut storage);
    list_all_mayors(Party::Republican, &mut storage);

    println!("\n\n================================================");
    println!("Warren Wilhelm Jr. is the real name of mayor Bill DeBlasio (Look it up)");
    println!("We already showed how to update a record, so rather than showing how to change his name, let's just delete him");

    let wwjr: Mayor = storage
        .get(Key::from(16))
        .expect("Could not find DeBlasio.  Check Brooklyn")
        .unwrap();

    storage
        .delete(&wwjr)
        .expect("I can't believe we STILL can't get rid of him");

    list_all_mayors(Party::Democrat, &mut storage);
}

fn list_all_mayors(with_affiliation: Party, storage: &mut Storage) {
    let query = storage.query::<Mayor>().expect("Could not build a query");
    let mayors = query.filter(|i| i.party == with_affiliation);

    println!("\n\nList of {:?} New York City Mayors", with_affiliation);
    println!("================================================");
    for mayor in mayors {
        println!("{:?}: {}", mayor.id, mayor.name);
    }
}
