use std::{
    collections::HashMap,
    sync::{Arc, PoisonError},
};

use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use itertools::Itertools;

use crate::{
    converter::{html2text, markdown2html, xml2html},
    database::{Database, is_not_found},
    domain::{
        ApiResult, Archive, Month, Post, PostsRequest, SmallPost, Storage, Tag, TagAggregate, Year,
    },
    paging::{page_offset, pages_count},
    typograph::typograph,
};

/// Posts per page in the public listing.
pub const PAGE_SIZE: i32 = 20;

/// Posts per page in the admin listing.
pub const ADMIN_PAGE_SIZE: i32 = 10;

const XML_PROLOG: &str = "<?xml version=\"1.0\"?>";

/// Result of looking up a public post by id.
pub enum Lookup<T = Article> {
    Found(T),
    /// Legacy id that now lives under a new id.
    Moved(i64),
    /// Missing post or draft.
    NotFound,
}

/// Public post prepared for reading.
pub struct Article {
    /// The post itself; its raw `text` is taken out after rendering.
    pub post: Post,
    /// Body rendered to typographed HTML.
    pub html: Arc<str>,
    /// Plain-text description for meta tags.
    pub description: Arc<str>,
}

/// Rendering markdown and typography is the costliest part of a read,
/// so its results are kept per post id.
#[derive(Default)]
struct RenderCache {
    articles: HashMap<i64, RenderedArticle>,
    teasers: HashMap<i64, Teaser>,
}

/// Valid while the post's `modified` stays the same.
#[derive(Clone)]
struct RenderedArticle {
    modified: DateTime<Utc>,
    html: Arc<str>,
    description: Arc<str>,
}

/// Valid while the raw teaser it was rendered from stays the same.
struct Teaser {
    markdown: bool,
    source: String,
    html: String,
}

/// Everything about posts: listing, reading, archive and editing.
///
/// The database lock is taken and released inside each call; rendering
/// happens outside the lock.
#[derive(Clone)]
pub struct Blog {
    db: Database,
    cache: Arc<std::sync::Mutex<RenderCache>>,
}

impl Blog {
    #[must_use]
    pub fn new(db: Database) -> Self {
        Self {
            db,
            cache: Arc::default(),
        }
    }

    /// Latest `limit` public posts.
    pub async fn recent(&self, limit: i32) -> Result<Vec<SmallPost>> {
        let page = self.list(limit, PostsRequest::default()).await?;
        Ok(page.result)
    }

    /// One page of public posts filtered by tag or period.
    /// An empty tag means no tag filter; drafts are never included.
    pub async fn page(&self, request: PostsRequest) -> Result<ApiResult<SmallPost>> {
        self.list(PAGE_SIZE, request).await
    }

    pub async fn article(&self, id: i64) -> Result<Lookup> {
        let stored = self
            .db
            .call(move |storage| {
                match storage.get_new_post_id(id) {
                    Ok(new_id) => return Ok(Lookup::Moved(new_id)),
                    Err(e) if is_not_found(&e) => {}
                    Err(e) => return Err(e),
                }
                match storage.get_post(id) {
                    Ok(post) if post.is_public => Ok(Lookup::Found(post)),
                    Ok(_) => Ok(Lookup::NotFound),
                    Err(e) if is_not_found(&e) => Ok(Lookup::NotFound),
                    Err(e) => Err(e),
                }
            })
            .await?;
        Ok(match stored {
            Lookup::Found(post) => Lookup::Found(self.render_article(post)?),
            Lookup::Moved(new_id) => Lookup::Moved(new_id),
            Lookup::NotFound => Lookup::NotFound,
        })
    }

    pub async fn archive(&self) -> Result<Archive> {
        let (tags, total, dates) = self
            .db
            .call(|storage| {
                Ok((
                    storage.get_aggregate_tags()?,
                    storage.count_posts(&PostsRequest::default())?,
                    storage.get_posts_create_dates()?,
                ))
            })
            .await?;

        Ok(Archive {
            tags: tag_levels(tags, total),
            years: group_to_years(&dates),
        })
    }

    /// Ids of all public posts, newest first.
    pub async fn ids(&self) -> Result<Vec<i64>> {
        self.db.call(|storage| storage.get_posts_ids()).await
    }

    /// One page of all posts, drafts included.
    pub async fn admin_page(&self, page: Option<i32>) -> Result<ApiResult<Post>> {
        let (page, offset) = page_offset(page, ADMIN_PAGE_SIZE);
        let (count, posts) = self
            .db
            .call(move |storage| {
                Ok((
                    storage.count_posts(&all_posts())?,
                    storage.get_posts(ADMIN_PAGE_SIZE, offset)?,
                ))
            })
            .await?;

        Ok(ApiResult {
            result: posts,
            pages: pages_count(count, ADMIN_PAGE_SIZE),
            page,
            count,
            status: "success",
        })
    }

    /// Raw post by id, drafts included.
    pub async fn draft(&self, id: i64) -> Result<Option<Post>> {
        self.db
            .call(move |storage| match storage.get_post(id) {
                Ok(post) => Ok(Some(post)),
                Err(e) if is_not_found(&e) => Ok(None),
                Err(e) => Err(e),
            })
            .await
    }

    /// Allocates the next free id, stores the post that `make` builds for it
    /// and returns that id.
    pub async fn create(&self, make: impl FnOnce(i64) -> Post + Send + 'static) -> Result<i64> {
        self.db
            .call(|storage| {
                let id = storage.next_post_id()?;
                storage.upsert_post(make(id))?;
                Ok(id)
            })
            .await
    }

    pub async fn update(&self, post: Post) -> Result<()> {
        let id = post.id;
        self.db.call(|storage| storage.upsert_post(post)).await?;
        self.evict(id);
        Ok(())
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        self.db.call(move |storage| storage.delete_post(id)).await?;
        self.evict(id);
        Ok(())
    }

    /// Number of all posts, drafts included.
    pub async fn count(&self) -> Result<i32> {
        self.db
            .call(|storage| storage.count_posts(&all_posts()))
            .await
    }

    async fn list(
        &self,
        page_size: i32,
        mut request: PostsRequest,
    ) -> Result<ApiResult<SmallPost>> {
        let (page, offset) = page_offset(request.page, page_size);
        request.include_private = None;
        request.tag = request.tag.filter(|tag| !tag.is_empty());

        let (count, posts) = self
            .db
            .call(move |storage| {
                Ok((
                    storage.count_posts(&request)?,
                    storage.get_small_posts(page_size, offset, &request)?,
                ))
            })
            .await?;

        Ok(ApiResult {
            result: self.render_teasers(posts),
            pages: pages_count(count, page_size),
            page,
            count,
            status: "success",
        })
    }

    fn render_article(&self, mut post: Post) -> Result<Article> {
        let text = std::mem::take(&mut post.text);
        let cached = self
            .cache()
            .articles
            .get(&post.id)
            .filter(|r| r.modified == post.modified)
            .cloned();
        let rendered = if let Some(rendered) = cached {
            rendered
        } else {
            let rendered = render_article(&post, text)?;
            self.cache().articles.insert(post.id, rendered.clone());
            rendered
        };

        Ok(Article {
            post,
            html: rendered.html,
            description: rendered.description,
        })
    }

    fn render_teasers(&self, mut posts: Vec<SmallPost>) -> Vec<SmallPost> {
        for post in &mut posts {
            let cached = self
                .cache()
                .teasers
                .get(&post.id)
                .filter(|t| t.markdown == post.markdown && t.source == post.short_text)
                .map(|t| t.html.clone());
            if let Some(html) = cached {
                post.short_text = html;
                continue;
            }
            let html = render_teaser(post.markdown, &post.short_text);
            let source = std::mem::replace(&mut post.short_text, html.clone());
            let teaser = Teaser {
                markdown: post.markdown,
                source,
                html,
            };
            self.cache().teasers.insert(post.id, teaser);
        }
        posts
    }

    fn evict(&self, id: i64) {
        let mut cache = self.cache();
        cache.articles.remove(&id);
        cache.teasers.remove(&id);
    }

    /// A poisoned lock still guards a usable cache: entries are inserted whole.
    fn cache(&self) -> std::sync::MutexGuard<'_, RenderCache> {
        self.cache.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn render_article(post: &Post, text: String) -> Result<RenderedArticle> {
    let html = if post.markdown {
        markdown2html(&text)?
    } else if text.starts_with(XML_PROLOG) {
        xml2html(&text)?
    } else {
        text
    };
    let body = if html.is_empty() {
        post.short_text.clone()
    } else {
        html
    };
    let html = typograph(&body)?;
    let description = describe(post, &html);

    Ok(RenderedArticle {
        modified: post.modified,
        html: html.into(),
        description: description.into(),
    })
}

fn describe(post: &Post, html: &str) -> String {
    if html.is_empty() {
        return post.title.clone();
    }
    let teaser = if post.markdown {
        markdown2html(&post.short_text).unwrap_or_default()
    } else {
        post.short_text.clone()
    };
    if teaser.is_empty() {
        post.title.clone()
    } else {
        html2text(&teaser).unwrap_or(teaser)
    }
}

fn all_posts() -> PostsRequest {
    PostsRequest {
        include_private: Some(true),
        ..Default::default()
    }
}

fn render_teaser(markdown: bool, source: &str) -> String {
    let html = if markdown {
        markdown2html(source).ok()
    } else {
        None
    };
    let html = html.as_deref().unwrap_or(source);
    typograph(html).unwrap_or_else(|_| html.to_owned())
}

/// Tag weight from 0 to 10 relative to the total number of posts.
fn tag_levels(tags: Vec<TagAggregate>, total_posts: i32) -> Vec<Tag> {
    tags.into_iter()
        .map(|tag| {
            let level = if total_posts > 0 {
                usize::try_from(tag.count * 10 / total_posts).unwrap_or_default()
            } else {
                0
            };
            Tag {
                title: tag.title,
                level,
            }
        })
        .collect()
}

fn group_to_years(dates: &[DateTime<Utc>]) -> Vec<Year> {
    dates
        .iter()
        .map(|dt| (dt.year(), dt.month()))
        .chunk_by(|(year, _month)| *year)
        .into_iter()
        .map(|(y, months)| {
            months
                .chunk_by(|(_y, m)| *m)
                .into_iter()
                .map(|(month, mg)| Month {
                    month: month.cast_signed(),
                    posts: i32::try_from(mg.count()).unwrap_or(i32::MAX),
                })
                .fold(Year::new(y), |mut y, m| {
                    y.append_month(m);
                    y
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::lock_test;
    use crate::sqlite::{Mode, Sqlite};
    use chrono::TimeZone;
    use rstest::{fixture, rstest};

    #[fixture]
    fn blog() -> Blog {
        let storage = Sqlite::open(":memory:", Mode::ReadWrite).unwrap();
        Blog::new(Database::new(storage))
    }

    fn post(id: i64, is_public: bool) -> Post {
        Post {
            id,
            title: format!("Post {id}"),
            short_text: format!("Teaser {id}"),
            text: format!("Text {id}"),
            is_public,
            created: Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()
                + chrono::Days::new(id.cast_unsigned()),
            ..Default::default()
        }
    }

    fn tagged(id: i64, tags: &[&str]) -> Post {
        Post {
            tags: tags.iter().map(ToString::to_string).collect(),
            ..post(id, true)
        }
    }

    fn created_at(id: i64, year: i32, month: u32) -> Post {
        Post {
            created: Utc.with_ymd_and_hms(year, month, 2, 0, 0, 0).unwrap(),
            ..post(id, true)
        }
    }

    async fn store(blog: &Blog, posts: Vec<Post>) {
        for p in posts {
            blog.update(p).await.unwrap();
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn write_waiting_for_database_lock_does_not_block_runtime() {
        // arrange
        let path = lock_test::temp_db("blog-lock");
        let storage = Sqlite::open(&path, Mode::ReadWrite).unwrap();
        let blog = Blog::new(Database::new(storage));

        // act
        let (stall, written) =
            lock_test::stall_while_locked(&path, async move { blog.update(post(1, true)).await })
                .await;

        // assert
        std::fs::remove_file(&path).unwrap();
        assert!(written.is_ok());
        assert!(
            stall < lock_test::LOCK_TIME / 2,
            "runtime was blocked for {stall:?}"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn page_skips_drafts(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true), post(2, false), post(3, true)]).await;

        // act
        let page = blog.page(PostsRequest::default()).await.unwrap();

        // assert
        let ids: Vec<i64> = page.result.iter().map(|p| p.id).collect();
        assert_eq!(vec![3, 1], ids);
        assert_eq!(2, page.count);
        assert_eq!(1, page.pages);
    }

    #[rstest]
    #[case(Some(1), 1, 20)]
    #[case(Some(2), 2, 5)]
    #[case(Some(0), 1, 20)]
    #[case(None, 1, 20)]
    #[tokio::test]
    async fn page_splits_by_page_size(
        blog: Blog,
        #[case] requested: Option<i32>,
        #[case] expected_page: i32,
        #[case] expected_len: usize,
    ) {
        // arrange
        store(&blog, (1..=25).map(|id| post(id, true)).collect()).await;
        let request = PostsRequest {
            page: requested,
            ..Default::default()
        };

        // act
        let page = blog.page(request).await.unwrap();

        // assert
        assert_eq!(expected_page, page.page);
        assert_eq!(expected_len, page.result.len());
        assert_eq!(2, page.pages);
        assert_eq!(25, page.count);
    }

    #[rstest]
    #[case(Some("rust"), vec![3, 1])]
    #[case(Some("zig"), vec![3, 2])]
    #[case(Some(""), vec![3, 2, 1])]
    #[case(None, vec![3, 2, 1])]
    #[tokio::test]
    async fn page_filters_by_tag(
        blog: Blog,
        #[case] tag: Option<&str>,
        #[case] expected: Vec<i64>,
    ) {
        // arrange
        store(
            &blog,
            vec![
                tagged(1, &["rust"]),
                tagged(2, &["zig"]),
                tagged(3, &["rust", "zig"]),
            ],
        )
        .await;
        let request = PostsRequest {
            tag: tag.map(ToOwned::to_owned),
            ..Default::default()
        };

        // act
        let page = blog.page(request).await.unwrap();

        // assert
        let ids: Vec<i64> = page.result.iter().map(|p| p.id).collect();
        assert_eq!(expected, ids);
        assert_eq!(i32::try_from(expected.len()).unwrap(), page.count);
    }

    #[rstest]
    #[case("**bold**", true, "<p><strong>bold</strong></p>\n")]
    #[case("a - **b**", true, "<p>a&nbsp;&mdash; <strong>b</strong></p>\n")]
    #[case("<p>a - b</p>", false, "<p>a&nbsp;&mdash; b</p>")]
    #[tokio::test]
    async fn page_renders_teasers(
        blog: Blog,
        #[case] short_text: &str,
        #[case] markdown: bool,
        #[case] expected: &str,
    ) {
        // arrange
        let p = Post {
            short_text: short_text.into(),
            markdown,
            ..post(1, true)
        };
        store(&blog, vec![p]).await;

        // act
        let page = blog.page(PostsRequest::default()).await.unwrap();

        // assert
        assert_eq!(expected, page.result[0].short_text);
    }

    #[rstest]
    #[tokio::test]
    async fn recent_returns_latest_public_posts(blog: Blog) {
        // arrange
        store(&blog, (1..=6).map(|id| post(id, id != 6)).collect()).await;

        // act
        let posts = blog.recent(3).await.unwrap();

        // assert
        let ids: Vec<i64> = posts.iter().map(|p| p.id).collect();
        assert_eq!(vec![5, 4, 3], ids);
    }

    #[rstest]
    #[case("Hello **world**", "", true, "<p>Hello <strong>world</strong></p>\n")]
    #[case("a - b", "", true, "<p>a&nbsp;&mdash; b</p>\n")]
    #[case("", "teaser only", true, "teaser only")]
    #[case("", "<p>a - b</p>", true, "<p>a&nbsp;&mdash; b</p>")]
    #[case("<p>a - b</p>", "", false, "<p>a&nbsp;&mdash; b</p>")]
    #[case(
        "<?xml version=\"1.0\"?><body><p>a - b</p><ul><li>c - d</li></ul></body>",
        "",
        false,
        "<p>a&nbsp;&mdash; b</p><ul><li>c&nbsp;&mdash; d</li></ul>"
    )]
    #[tokio::test]
    async fn article_renders_body(
        blog: Blog,
        #[case] text: &str,
        #[case] short_text: &str,
        #[case] markdown: bool,
        #[case] expected: &str,
    ) {
        // arrange
        let p = Post {
            text: text.into(),
            short_text: short_text.into(),
            markdown,
            ..post(1, true)
        };
        store(&blog, vec![p]).await;

        // act
        let lookup = blog.article(1).await.unwrap();

        // assert
        let Lookup::Found(article) = lookup else {
            panic!("article not found");
        };
        assert_eq!(expected, &*article.html);
        assert!(article.post.text.is_empty());
    }

    #[rstest]
    #[case("**Short** teaser", true, "Short teaser")]
    #[case("<p>Html teaser</p>", false, "Html teaser")]
    #[case("", false, "Post 1")]
    #[tokio::test]
    async fn article_describes_itself_by_teaser_or_title(
        blog: Blog,
        #[case] short_text: &str,
        #[case] markdown: bool,
        #[case] expected: &str,
    ) {
        // arrange
        let p = Post {
            short_text: short_text.into(),
            markdown,
            ..post(1, true)
        };
        store(&blog, vec![p]).await;

        // act
        let lookup = blog.article(1).await.unwrap();

        // assert
        let Lookup::Found(article) = lookup else {
            panic!("article not found");
        };
        assert_eq!(expected, article.description.trim());
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    #[tokio::test]
    async fn article_hides_drafts_and_missing_posts(blog: Blog, #[case] stored_as_draft: bool) {
        // arrange
        if stored_as_draft {
            store(&blog, vec![post(1, false)]).await;
        }

        // act
        let lookup = blog.article(1).await.unwrap();

        // assert
        assert!(matches!(lookup, Lookup::NotFound));
    }

    #[rstest]
    #[tokio::test]
    async fn article_follows_moved_posts(blog: Blog) {
        // arrange
        store(&blog, vec![post(42, true)]).await;
        blog.db
            .call(|storage| {
                Ok(storage.execute("INSERT INTO post_remap (old_id, post_id) VALUES (7, 42)")?)
            })
            .await
            .unwrap();

        // act
        let lookup = blog.article(7).await.unwrap();

        // assert
        assert!(matches!(lookup, Lookup::Moved(42)));
    }

    #[rstest]
    #[tokio::test]
    async fn article_rendered_again_after_update(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true)]).await;
        blog.article(1).await.unwrap();
        let updated = Post {
            text: "<p>Updated</p>".into(),
            short_text: "<p>Updated teaser</p>".into(),
            ..post(1, true)
        };
        store(&blog, vec![updated]).await;

        // act
        let lookup = blog.article(1).await.unwrap();

        // assert
        let Lookup::Found(article) = lookup else {
            panic!("article not found");
        };
        assert_eq!("<p>Updated</p>", &*article.html);
        assert_eq!("Updated teaser", article.description.trim());
    }

    #[rstest]
    #[tokio::test]
    async fn article_rendered_once_returns_same_body(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true)]).await;
        let Lookup::Found(first) = blog.article(1).await.unwrap() else {
            panic!("article not found");
        };

        // act
        let Lookup::Found(second) = blog.article(1).await.unwrap() else {
            panic!("article not found");
        };

        // assert
        assert!(Arc::ptr_eq(&first.html, &second.html));
        assert_eq!(first.post.title, second.post.title);
    }

    #[rstest]
    #[case(false, "<p>Changed</p>", "<p>Changed</p>")]
    #[case(true, "**Changed**", "<p><strong>Changed</strong></p>")]
    #[tokio::test]
    async fn page_renders_teaser_again_after_update(
        blog: Blog,
        #[case] markdown: bool,
        #[case] short_text: &str,
        #[case] expected: &str,
    ) {
        // arrange
        store(&blog, vec![post(1, true)]).await;
        blog.page(PostsRequest::default()).await.unwrap();
        let updated = Post {
            short_text: short_text.into(),
            markdown,
            ..post(1, true)
        };
        store(&blog, vec![updated]).await;

        // act
        let page = blog.page(PostsRequest::default()).await.unwrap();

        // assert
        assert_eq!(expected, page.result[0].short_text.trim());
    }

    #[rstest]
    #[tokio::test]
    async fn archive_groups_public_posts_by_year_and_month(blog: Blog) {
        // arrange
        store(
            &blog,
            vec![
                created_at(1, 2015, 2),
                created_at(2, 2015, 2),
                created_at(3, 2015, 3),
                created_at(4, 2016, 3),
                Post {
                    is_public: false,
                    ..created_at(5, 2017, 1)
                },
            ],
        )
        .await;

        // act
        let archive = blog.archive().await.unwrap();

        // assert
        let years: Vec<(i32, i32, usize)> = archive
            .years
            .iter()
            .map(|y| (y.year, y.posts, y.months.len()))
            .collect();
        assert_eq!(vec![(2016, 1, 1), (2015, 3, 2)], years);
    }

    #[rstest]
    #[tokio::test]
    async fn archive_weighs_tags_by_share_of_posts(blog: Blog) {
        // arrange
        store(
            &blog,
            vec![
                tagged(1, &["rust"]),
                tagged(2, &["rust"]),
                tagged(3, &["rust", "zig"]),
                tagged(4, &[]),
            ],
        )
        .await;

        // act
        let archive = blog.archive().await.unwrap();

        // assert
        let mut tags: Vec<(String, usize)> = archive
            .tags
            .into_iter()
            .map(|t| (t.title, t.level))
            .collect();
        tags.sort();
        assert_eq!(
            vec![(String::from("rust"), 7), (String::from("zig"), 2)],
            tags
        );
    }

    #[rstest]
    #[tokio::test]
    async fn ids_lists_public_posts_newest_first(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true), post(2, false), post(3, true)]).await;

        // act
        let ids = blog.ids().await.unwrap();

        // assert
        assert_eq!(vec![3, 1], ids);
    }

    #[rstest]
    #[tokio::test]
    async fn admin_page_includes_drafts(blog: Blog) {
        // arrange
        store(&blog, (1..=12).map(|id| post(id, id % 2 == 0)).collect()).await;

        // act
        let page = blog.admin_page(Some(2)).await.unwrap();

        // assert
        let ids: Vec<i64> = page.result.iter().map(|p| p.id).collect();
        assert_eq!(vec![2, 1], ids);
        assert_eq!(12, page.count);
        assert_eq!(2, page.pages);
        assert_eq!(2, page.page);
    }

    #[rstest]
    #[tokio::test]
    async fn create_assigns_next_free_id(blog: Blog) {
        // arrange
        let make = |id| post(id, false);

        // act
        let first = blog.create(make).await.unwrap();
        let second = blog.create(make).await.unwrap();

        // assert
        assert_eq!((1, 2), (first, second));
        assert_eq!("Post 2", blog.draft(2).await.unwrap().unwrap().title);
    }

    #[rstest]
    #[tokio::test]
    async fn update_replaces_stored_post(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, false)]).await;
        let changed = Post {
            title: "Changed".into(),
            ..post(1, false)
        };

        // act
        blog.update(changed).await.unwrap();

        // assert
        assert_eq!("Changed", blog.draft(1).await.unwrap().unwrap().title);
    }

    #[rstest]
    #[tokio::test]
    async fn delete_removes_post(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true)]).await;

        // act
        blog.delete(1).await.unwrap();

        // assert
        assert!(blog.draft(1).await.unwrap().is_none());
        assert_eq!(0, blog.count().await.unwrap());
    }

    #[rstest]
    #[tokio::test]
    async fn count_includes_drafts(blog: Blog) {
        // arrange
        store(&blog, vec![post(1, true), post(2, false)]).await;

        // act
        let count = blog.count().await.unwrap();

        // assert
        assert_eq!(2, count);
    }

    #[rstest]
    #[case(0, 0, 0)]
    #[case(3, 0, 0)]
    #[case(0, 10, 0)]
    #[case(1, 10, 1)]
    #[case(3, 4, 7)]
    #[case(1, 11, 0)]
    #[case(10, 10, 10)]
    fn tag_levels_tests(#[case] count: i32, #[case] total: i32, #[case] expected: usize) {
        // arrange
        let tags = vec![TagAggregate {
            title: String::from("rust"),
            count,
        }];

        // act
        let actual = tag_levels(tags, total);

        // assert
        assert_eq!(expected, actual[0].level);
    }
}
