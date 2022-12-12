use std::path::PathBuf;

use crate::{
    converter::markdown2html,
    domain::{
        ApiResult, Archive, Month, PostsRequest, SmallPost, Storage, Tag, TagAggregate, Year,
    },
    sqlite::{Mode, Sqlite},
};
use chrono::{DateTime, Datelike, Utc};
use itertools::Itertools;

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

const MONTHS: &[&str] = &[
    "Январь",
    "Февраль",
    "Март",
    "Апрель",
    "Май",
    "Июнь",
    "Июль",
    "Август",
    "Сентябрь",
    "Октябрь",
    "Ноябрь",
    "Декабрь",
];

pub fn archive(storage_path: PathBuf) -> Archive {
    let storage = Sqlite::open(storage_path, Mode::ReadOnly).unwrap();
    let aggregated_tags: Vec<TagAggregate> = storage.get_aggregate_tags().unwrap();
    let req = PostsRequest {
        ..Default::default()
    };
    let total_posts = storage.count_posts(req).unwrap();
    let dates: Vec<DateTime<Utc>> = storage.get_posts_create_dates().unwrap();

    let years = group_by_years(dates);

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

    Archive { tags, years }
}

fn group_by_years(dates: Vec<DateTime<Utc>>) -> Vec<Year> {
    let ygrp = dates
        .iter()
        .map(|dt| (dt.year(), dt.month()))
        .group_by(|(year, _month)| *year);
    let mut result = vec![];
    for (k, g) in &ygrp {
        let mgrp = g.group_by(|(_y, m)| *m);

        let mut months = vec![];
        let mut posts_in_year = 0;
        for (k, mg) in &mgrp {
            let m = Month {
                month: k as i32,
                posts: mg.count() as i32,
                name: MONTHS[k as usize - 1].to_string(),
            };
            posts_in_year += m.posts;
            months.push(m)
        }

        let year = Year {
            year: k,
            posts: posts_in_year,
            months,
        };
        result.push(year);
    }
    result
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
    use chrono::NaiveDate;
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

    #[test]
    fn group_by_years_tests() {
        // arrange
        let dt1 = NaiveDate::from_ymd_opt(2015, 2, 2)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt1 = DateTime::<Utc>::from_local(dt1, Utc);

        let dt2 = NaiveDate::from_ymd_opt(2015, 2, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt2 = DateTime::<Utc>::from_local(dt2, Utc);

        let dt3 = NaiveDate::from_ymd_opt(2015, 3, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt3 = DateTime::<Utc>::from_local(dt3, Utc);

        let dt4 = NaiveDate::from_ymd_opt(2016, 3, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap();
        let dt4 = DateTime::<Utc>::from_local(dt4, Utc);

        let dates = vec![dt1, dt2, dt3, dt4];

        // act
        let actual = group_by_years(dates);

        // assert
        assert_eq!(2, actual.len());
        assert_eq!(3, actual[0].posts);
        assert_eq!(1, actual[1].posts);
    }
}
