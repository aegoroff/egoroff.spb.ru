use std::path::PathBuf;

use chrono::{DateTime, Utc, Datelike};
use itertools::Itertools;
use crate::{
    converter::markdown2html,
    domain::{ApiResult, Archive, PostsRequest, SmallPost, Storage, Tag, TagAggregate},
    sqlite::{Mode, Sqlite},
};

const RANKS: &[&str] = &[
    "tagRank10",
    "tagRank9",
    "tagRank8",
    "tagRank7",
    "tagRank6",
    "tagRank5",
    "tagRank4",
    "tagRank3",
    "tagRank2",
    "tagRank1",
];

pub fn archive(storage_path: PathBuf) -> Archive {
    let storage = Sqlite::open(storage_path, Mode::ReadOnly).unwrap();
    let aggregated_tags: Vec<TagAggregate> = storage.get_aggregate_tags().unwrap();
    let req = PostsRequest {
        ..Default::default()
    };
    let total_posts = storage.count_posts(req).unwrap();
    let dates : Vec<DateTime<Utc>> = storage.get_posts_create_dates().unwrap();

    dates.iter().map(|dt| (dt.year(), dt.month())).group_by(|(year, month)| *year);

    let tags = aggregated_tags
        .iter()
        .map(|tag| {
            let ix = (tag.count as f32 / total_posts as f32 * 10.0) as usize;
            Tag {
                title: tag.title.clone(),
                level: RANKS[ix].to_string(),
            }
        })
        .collect();

    Archive {
        tags,
        ..Default::default()
    }
}

pub fn get_posts(storage_path: PathBuf, page_size: i32, request: PostsRequest) -> ApiResult {
    let storage = Sqlite::open(storage_path, Mode::ReadOnly).unwrap();

    let page = request.page.unwrap_or(1);

    let total_posts_count = storage.count_posts(request.clone()).unwrap();
    let pages_count = count_pages(total_posts_count, page_size);

    let posts = storage
        .get_small_posts(page_size, page_size * (page - 1), request)
        .unwrap();

    ApiResult {
        result: update_short_text(posts),
        pages: pages_count,
        page,
        count: total_posts_count,
        status: "success".to_string(),
    }
}

fn count_pages(count: i32, page_size: i32) -> i32 {
    count / page_size + i32::from(count % page_size > 0)
}

fn update_short_text(posts: Vec<SmallPost>) -> Vec<SmallPost> {
    let posts: Vec<SmallPost> = posts
        .into_iter()
        .map(|mut post| {
            if post.markdown {
                if let Ok(text) = markdown2html(&post.short_text) {
                    post.short_text = text
                }
            }
            post
        })
        .collect();
    posts
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(0, 0)]
    #[case(20, 1)]
    #[case(21, 2)]
    #[case(40, 2)]
    #[case(41, 3)]
    #[case(60, 3)]
    #[case(66, 4)]
    fn count_pages_tests(#[case] count: i32, #[case] expected: i32) {
        // arrange
        let page_size = 20;

        // act
        let actual = count_pages(count, page_size);

        // assert
        assert_eq!(expected, actual);
    }
}
