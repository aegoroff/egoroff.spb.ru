use chrono::{DateTime, Utc};
use kernel::{
    domain::{Post, Storage},
    sqlite::{Mode, Sqlite},
};
use tokio::fs;

#[macro_use]
extern crate serde;

#[derive(Deserialize, Default, Debug)]
pub struct MigrateResult {
    pub result: Vec<Post>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Tags {
    pub values: Vec<Value>,
}

#[derive(Deserialize, Default, Debug)]
pub struct Value {
    #[serde(rename(deserialize = "stringValue"))]
    pub string_value: String,
}

pub async fn run(uri: &str, db_path: &str, from_file: bool) {
    if from_file {
        from_files(uri, db_path).await;
    } else {
        from_uri(uri, db_path).await
    }
}

async fn from_files(files_path: &str, db_path: &str) {
    let mut dir = tokio::fs::read_dir(files_path).await.unwrap();
    let mut storage = Sqlite::open(db_path, Mode::ReadWrite).unwrap();
    loop {
        let entry = dir.next_entry().await;
        if entry.is_err() {
            break;
        }
        let entry = entry.unwrap();
        if entry.is_none() {
            break;
        }
        let entry = entry.unwrap();

        let path = entry.path();
        let path = path.as_path();

        if let Some(e) = path.extension() {
            if let Some(ext) = e.to_str() {
                if ext != "txt" {
                    continue;
                }
                let title = path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .strip_suffix(".txt")
                    .unwrap();
                let title = String::from(title);

                let contents = fs::read(entry.path()).await.unwrap();
                let contents = String::from_utf8(contents).unwrap();
                let mut parts = contents.split(
                    "--------------------------------------------------------------------------",
                );
                let id: i64 = parts.next().unwrap().trim().parse().unwrap();
                let date = parts.next().unwrap().trim();
                let datetime = DateTime::parse_from_rfc3339(date).unwrap();
                let short_text = parts.next().unwrap().trim();
                let text = parts.next().unwrap().trim();
                let tags = parts.next().unwrap().trim();
                let tags: Tags = serde_json::from_str(tags).unwrap();
                let post = Post {
                    created: datetime.with_timezone(&Utc),
                    modified: datetime.with_timezone(&Utc),
                    id,
                    title,
                    short_text: short_text.to_string(),
                    text: text.to_string(),
                    markdown: true,
                    is_public: true,
                    tags: tags.values.iter().map(|v| v.string_value.clone()).collect(),
                };
                storage.upsert_post(post).unwrap();
            }
        }
    }
}

async fn from_uri(uri: &str, db_path: &str) {
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
