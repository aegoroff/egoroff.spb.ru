use anyhow::Context;
use serde::Deserialize;

use crate::{
    body::Redirect,
    domain::{Apache, Downloadable, FilesContainer},
};

use super::*;

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

const PORTFOLIO_PATH: &str = "/portfolio/";

#[derive(Deserialize, Default)]
pub struct StoredFile {
    pub id: i64,
    pub path: String,
    pub size: u64,
}

#[derive(Template)]
#[template(path = "portfolio/index.html")]
struct Portfolio<'a> {
    html_class: &'a str,
    title: &'a str,
    title_path: &'a str,
    keywords: &'a str,
    meta_description: &'a str,
    flashed_messages: Vec<Message>,
    downloads: Vec<FilesContainer>,
    apache_docs: Vec<Apache>,
}

pub async fn serve_index(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let Some(section) = page_context.site_graph.get_section("portfolio") else {
        return make_500_page();
    };

    let title_path = page_context.site_graph.make_title_path(PORTFOLIO_PATH);

    let mut context = Portfolio {
        html_class: "portfolio",
        title: &section.title,
        title_path: &title_path,
        keywords: get_keywords(section),
        meta_description: &section.descr,
        flashed_messages: vec![],
        downloads: vec![],
        apache_docs: vec![],
    };

    let downloads = read_downloads(page_context.clone()).await;
    if let Some(downloads) = downloads {
        context.downloads = downloads;
    }

    match read_apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.apache_docs = docs;
            serve_page(context)
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page()
        }
    }
}

#[derive(Template)]
#[template(path = "portfolio/apache.html")]
struct ApacheDocument<'a> {
    html_class: &'a str,
    title: &'a str,
    title_path: &'a str,
    keywords: &'a str,
    meta_description: &'a str,
    flashed_messages: Vec<Message>,
    content: &'a str,
}

pub async fn serve_apache_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let apache_documents = match read_apache_documents(&page_context.base_path) {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            return make_500_page();
        }
    };

    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let Some(doc) = map.get(doc) else {
        return make_404_page();
    };

    let uri = format!("{PORTFOLIO_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    let asset = ApacheTemplates::get(&path);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        let context = ApacheDocument {
            html_class: "",
            title: &doc.title,
            title_path: &title_path,
            keywords: &doc.keywords,
            meta_description: &doc.description,
            flashed_messages: vec![],
            content: &content,
        };
        serve_page(context)
    } else {
        make_404_page()
    }
}

pub fn read_apache_documents(base_path: &Path) -> Result<Vec<crate::domain::Apache>> {
    let config_path = base_path.join("apache/config.json");
    let file = File::open(config_path)
        .with_context(|| "Failed to open file that contain Apache documents configuration")?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)
        .with_context(|| "Failed to deserialize Apache documents configuration")?;
    Ok(result)
}

pub async fn redirect_to_real_document(
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let new_path = format!("/portfolio/{path}");
    Redirect::permanent(&new_path)
}

async fn read_downloads(page_context: Arc<PageContext<'_>>) -> Option<Vec<FilesContainer>> {
    let storage = page_context.storage.lock().await;

    let folders = storage.get_folders().ok()?;

    let mut result = vec![];
    for f in folders {
        let mut resource = Resource::new(&page_context.store_uri)?;
        let mut container = FilesContainer {
            title: f.title,
            ..Default::default()
        };
        resource.append_path("api").append_path(&f.bucket);
        let client = Client::new();
        if let Ok(r) = client.get(resource.to_string()).send().await {
            let files = r.json::<Vec<StoredFile>>().await;
            if let Ok(files) = files {
                for file in files {
                    let meta_info = storage.get_download(file.id).ok()?;
                    let downloadable = Downloadable {
                        title: meta_info.title,
                        path: format!("/storage/{}/{}", f.bucket, file.path),
                        filename: file.path,
                        size: file.size,
                    };
                    container.files.push(downloadable);
                }
            }
        }
        result.push(container);
    }
    Some(result)
}
