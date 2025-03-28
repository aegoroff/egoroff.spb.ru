use askama::Template;
use kernel::domain::{Post, SmallPost};

use crate::domain::{Apache, BlogRequest, Config, Error, FilesContainer, Message, Poster};

#[derive(Template, Default)]
#[template(path = "error.html")]
pub struct ErrorPage<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub error: Error,
}

#[derive(Template, Default)]
#[template(path = "welcome.html")]
pub struct Index<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub posts: Vec<SmallPost>,
    pub apache_docs: Vec<crate::domain::Apache>,
    pub flashed_messages: Vec<Message>,
}

#[derive(Template)]
#[template(path = "search.html")]
pub struct Search<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub config: &'a Config,
}

#[derive(Template)]
#[template(path = "portfolio/apache.html")]
pub struct ApacheDocument<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "portfolio/index.html")]
pub struct Portfolio<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub downloads: Vec<FilesContainer>,
    pub apache_docs: Vec<Apache>,
}

#[derive(Template)]
#[template(path = "blog/index.html")]
pub struct BlogIndex<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub poster: &'a Poster<SmallPost>,
    pub request: &'a BlogRequest,
}

#[derive(Template)]
#[template(path = "blog/post.html")]
pub struct BlogPost<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: String,
    pub flashed_messages: Vec<Message>,
    pub main_post: &'a Post,
    pub content: &'a str,
}

#[derive(Template, Default)]
#[template(path = "signin.html")]
pub struct Signin<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
    pub google_signin_url: &'a str,
    pub github_signin_url: &'a str,
    pub yandex_signin_url: &'a str,
}

#[derive(Template, Default)]
#[template(path = "profile.html")]
pub struct Profile<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub flashed_messages: Vec<Message>,
}

#[derive(Template, Default)]
#[template(path = "admin.html")]
pub struct Admin<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
}

mod filters {
    use kernel::typograph;

    pub fn typograph<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
        let s = s.to_string();
        match typograph::typograph(&s) {
            Ok(s) => Ok(s),
            Err(_) => Err(::askama::Error::Fmt),
        }
    }
}
