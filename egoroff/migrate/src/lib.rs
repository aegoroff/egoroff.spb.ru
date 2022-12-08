use kernel::{
    domain::{Post, Storage},
    sqlite::{Mode, Sqlite},
};

#[macro_use]
extern crate serde;

#[derive(Deserialize, Default, Debug)]
pub struct MigrateResult {
    pub result: Vec<Post>,
}

pub async fn run(uri: &str, db_path: &str) {
    let body = reqwest::get(uri)
        .await
        .unwrap()
        .json::<MigrateResult>()
        .await
        .unwrap();

    let mut storage = Sqlite::open(db_path, Mode::ReadWrite).unwrap();
    match storage.new_database() {
        Ok(()) => {
            println!("Success");
        }
        Err(e) => {
            println!("{e:#?}");
            return;
        }
    }
    for post in body.result {
        let id = post.id;
        match storage.upsert_post(post) {
            Ok(()) => println!("Inserted: {id}"),
            Err(e) => println!("Insertion error: {e:#?}"),
        }
    }
}
