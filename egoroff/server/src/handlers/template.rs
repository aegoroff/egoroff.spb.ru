use askama::Template;
use axum::http::{self, StatusCode};
use axum::response::{IntoResponse, Response};
use kernel::domain::{Post, SmallPost};

use crate::domain::{Apache, BlogRequest, Config, Error, Message, Poster};

fn text_html_respose<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(body) => {
            let headers = [
                (
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("text/html"),
                ),
                (
                    http::header::X_XSS_PROTECTION,
                    http::HeaderValue::from_static("1; mode=block"),
                ),
                (
                    http::header::X_CONTENT_TYPE_OPTIONS,
                    http::HeaderValue::from_static("nosniff"),
                ),
                (
                    http::header::X_FRAME_OPTIONS,
                    http::HeaderValue::from_static("sameorigin"),
                ),
                (
                    http::header::CONTENT_SECURITY_POLICY,
                    http::HeaderValue::from_static(
                        "default-src 'none'; script-src 'self'; frame-ancestors 'self'; connect-src 'self' www.googleapis.com; img-src 'self' data: *.ggpht.com avatars.githubusercontent.com *.googleusercontent.com i.imgur.com; style-src 'self' 'unsafe-inline' fonts.googleapis.com; font-src 'self' fonts.googleapis.com fonts.gstatic.com;",
                    ),
                ),
                (
                    http::header::REFERRER_POLICY,
                    http::HeaderValue::from_static("strict-origin-when-cross-origin"),
                ),
            ];

            (headers, body).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

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
    pub year: u32,
}

impl IntoResponse for ErrorPage<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for Index<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for Search<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for ApacheDocument<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub apache_docs: Vec<Apache>,
    pub year: u32,
}

impl IntoResponse for Portfolio<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for BlogIndex<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for BlogPost<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for Signin<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
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
    pub year: u32,
}

impl IntoResponse for Profile<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
}

#[derive(Template, Default)]
#[template(path = "admin.html")]
pub struct Admin<'a> {
    pub html_class: &'a str,
    pub title: &'a str,
    pub title_path: &'a str,
    pub keywords: &'a str,
    pub meta_description: &'a str,
    pub year: u32,
}

impl IntoResponse for Admin<'_> {
    fn into_response(self) -> axum::response::Response {
        text_html_respose(self)
    }
}

mod filters {
    use kernel::typograph as kernel_typograph;

    #[askama::filter_fn]
    pub fn typograph<T: std::fmt::Display>(
        s: T,
        _: &dyn askama::Values,
    ) -> ::askama::Result<String> {
        let s = s.to_string();
        match kernel_typograph::typograph(&s) {
            Ok(s) => Ok(s),
            Err(_) => Err(::askama::Error::Fmt),
        }
    }
}
