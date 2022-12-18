use super::*;

#[derive(RustEmbed)]
#[folder = "../../templates/apache"]
struct ApacheTemplates;

pub async fn serve_portfolio(
    Extension(page_context): Extension<Arc<PageContext>>,
) -> impl IntoResponse {
    let section = page_context.site_graph.get_section("portfolio").unwrap();

    let uri = page_context.site_graph.full_path("portfolio");
    let title_path = page_context.site_graph.make_title_path(&uri);

    let mut context = Context::new();
    context.insert("html_class", "portfolio");
    context.insert(TITLE_KEY, &section.title);
    context.insert("title_path", &title_path);
    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("keywords", &section.keywords);
    context.insert("meta_description", &section.descr);
    context.insert("config", &page_context.site_config);

    match apache_documents(&page_context.base_path) {
        Ok(docs) => {
            context.insert("apache_docs", &docs);
            (
                StatusCode::OK,
                serve_page(&context, "portfolio/index.html", &page_context.tera),
            )
        }
        Err(e) => {
            tracing::error!("{e:#?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                make_500_page(&mut context, &page_context.tera),
            )
        }
    }
}

pub async fn serve_portfolio_document(
    Extension(page_context): Extension<Arc<PageContext>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();

    let messages: Vec<String> = Vec::new();
    context.insert("flashed_messages", &messages);
    context.insert("gin_mode", MODE);
    context.insert("html_class", "");
    context.insert("config", &page_context.site_config);

    let apache_documents = match apache_documents(&page_context.base_path) {
        Ok(docs) => docs,
        Err(e) => {
            tracing::error!("{e:#?}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                make_500_page(&mut context, &page_context.tera),
            );
        }
    };

    let map: HashMap<&str, &crate::domain::Apache> = apache_documents
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();

    let doc = path.trim_end_matches(".html");

    let doc = match map.get(doc) {
        Some(item) => item,
        None => {
            return (
                StatusCode::NOT_FOUND,
                make_404_page(&mut context, &page_context.tera),
            )
        }
    };

    let uri = page_context.site_graph.full_path("portfolio");
    let uri = format!("{uri}/{path}");
    let title_path = page_context.site_graph.make_title_path(&uri);

    context.insert(TITLE_KEY, &doc.title);
    context.insert("title_path", &title_path);
    context.insert("keywords", &doc.keywords);
    context.insert("meta_description", &doc.description);

    let asset = ApacheTemplates::get(&path);
    if let Some(file) = asset {
        let content = String::from_utf8_lossy(&file.data);
        context.insert("content", &content);
        (
            StatusCode::OK,
            serve_page(&context, "portfolio/apache.html", &page_context.tera),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            make_404_page(&mut context, &page_context.tera),
        )
    }
}

pub fn apache_documents(base_path: &Path) -> Result<Vec<crate::domain::Apache>> {
    let config_path = base_path.join("apache/config.json");
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader)?;
    Ok(result)
}