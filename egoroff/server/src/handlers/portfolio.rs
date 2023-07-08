use crate::body::Redirect;

use super::*;

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

const PORTFOLIO_PATH: &str = "/portfolio/";

pub async fn serve_index(State(page_context): State<Arc<PageContext<'_>>>) -> impl IntoResponse {
    let mut context = Context::new();
    let Some(section) = page_context.site_graph.get_section("portfolio") else { return make_500_page(&mut context, &page_context.tera) };

    let title_path = page_context.site_graph.make_title_path(PORTFOLIO_PATH);

    context.insert(HTML_CLASS_KEY, "portfolio");
    context.insert(TITLE_KEY, &section.title);
    context.insert(TITLE_PATH_KEY, &title_path);
    context.insert(KEYWORDS_KEY, &section.keywords);
    context.insert(META_DESCR_KEY, &section.descr);
    context.insert(CONFIG_KEY, &page_context.site_config);

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
    let mut context = Context::new();
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
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}

pub async fn redirect_to_real_document(
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let new_path = format!("/portfolio/{path}");
    Redirect::permanent(&new_path)
}
