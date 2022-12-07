use kernel::domain::Post;

#[macro_use]
extern crate serde;

#[derive(Deserialize, Default, Debug)]
pub struct MigrateResult {
    pub result: Vec<Post>,
}

pub async fn run(uri: &str) {
    let body = reqwest::get(uri)
        .await
        .unwrap()
        .json::<MigrateResult>()
        .await
        .unwrap();

    println!("body = {:#?}", body);
}
