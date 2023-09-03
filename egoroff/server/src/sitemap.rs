use anyhow::Result;
use kernel::xml::Builder;

const SITE: &str = "https://www.egoroff.spb.ru/";
const URLSET_ELT: &str = "urlset";
const URL_ELT: &str = "url";
const LOC_ELT: &str = "loc";
const CHANGE_FREQ_ELT: &str = "changefreq";
const PRIORITY_ELT: &str = "priority";

struct Url<'a> {
    pub location: &'a str,
    pub changefreq: &'a str,
    pub priority: &'a str,
}

pub fn make_site_map(
    apache_docs: Vec<crate::domain::Apache>,
    post_ids: Vec<i64>,
) -> Result<String> {
    let mut builder = Builder::new();

    builder.write_attributed_start_tag(
        URLSET_ELT,
        vec![("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")],
    )?;

    let root = Url {
        location: SITE,
        changefreq: "weekly",
        priority: "1.0",
    };
    write_url(&mut builder, &root)?;

    let blog = Url {
        location: &format!("{SITE}blog/"),
        changefreq: "weekly",
        priority: "0.7",
    };
    write_url(&mut builder, &blog)?;

    let porfolio = Url {
        location: &format!("{SITE}porfolio/"),
        changefreq: "weekly",
        priority: "0.7",
    };
    write_url(&mut builder, &porfolio)?;

    for doc in apache_docs {
        let d = Url {
            location: &format!("{SITE}porfolio/{}.html", doc.id),
            changefreq: "yearly",
            priority: "1.0",
        };
        write_url(&mut builder, &d)?;
    }

    for id in post_ids {
        let d = Url {
            location: &format!("{SITE}blog/{id}.html"),
            changefreq: "yearly",
            priority: "1.0",
        };
        write_url(&mut builder, &d)?;
    }

    builder.write_end_tag(URLSET_ELT)?;
    builder.to_string()
}

fn write_url(builder: &mut Builder, root: &Url) -> Result<()> {
    builder.write_start_tag(URL_ELT)?;
    builder.write_element(LOC_ELT, root.location)?;
    builder.write_element(CHANGE_FREQ_ELT, root.changefreq)?;
    builder.write_element(PRIORITY_ELT, root.priority)?;
    builder.write_end_tag(URL_ELT)?;
    Ok(())
}
