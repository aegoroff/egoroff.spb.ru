use anyhow::Context;
use serde::Deserialize;

use crate::{
    body::Redirect,
    domain::{Downloadable, FilesContainer},
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

pub async fn serve_index(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let mut context = tera::Context::new();
    let Some(section) = page_context.site_graph.get_section("portfolio") else { return make_500_page(&mut context, &page_context.tera) };

    let title_path = page_context.site_graph.make_title_path(PORTFOLIO_PATH);

    context.insert(HTML_CLASS_KEY, "portfolio");
    context.insert(TITLE_KEY, &section.title);
    context.insert(TITLE_PATH_KEY, &title_path);
    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert(META_DESCR_KEY, &section.descr);
    context.insert(CONFIG_KEY, &page_context.site_config);

    let downloads = read_downloads(page_context.clone()).await;
    if let Some(downloads) = downloads {
        context.insert("downloads", &downloads);
    }

    match read_apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.insert(APACHE_DOCS_KEY, &docs);
            serve_page(&context, "portfolio/index.html", &page_context.tera)
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            make_500_page(&mut context, &page_context.tera)
        }
    }
}

pub async fn serve_apache_document(
    State(page_context): State<Arc<PageContext<'_>>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let mut context = tera::Context::new();
    context.insert(CONFIG_KEY, &page_context.site_config);

    let apache_documents = match read_apache_documents(&page_context.base_path) {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            return make_500_page(&mut context, &page_context.tera);
        }
    };

    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let Some(doc) = map.get(doc) else {
        return make_404_page(&mut context, &page_context.tera);
    };

    let uri = format!("{PORTFOLIO_PATH}{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &doc.title);
    context.insert(TITLE_PATH_KEY, &title_path);
    context.insert(KEYWORDS_KEY, &doc.keywords);
    context.insert(META_DESCR_KEY, &doc.description);

    let asset = ApacheTemplates::get(&path);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        context.insert("content", &content);
        serve_page(&context, "portfolio/apache.html", &page_context.tera)
    } else {
        make_404_page(&mut context, &page_context.tera)
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
