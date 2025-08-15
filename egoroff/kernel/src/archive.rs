use crate::{
    converter::markdown2html,
    domain::{
        ApiResult, Archive, Month, Post, PostsRequest, SmallPost, Storage, Tag, TagAggregate, Year,
    },
    sqlite::Sqlite,
};
use anyhow::Result;
use chrono::{DateTime, Datelike, Utc};
use futures::lock::MutexGuard;
use itertools::Itertools;

pub fn archive(storage: MutexGuard<Sqlite>) -> Result<Archive> {
    let aggregated_tags: Vec<TagAggregate> = storage.get_aggregate_tags()?;
    let req = PostsRequest {
        ..Default::default()
    };
    let total_posts = storage.count_posts(req)?;
    let dates: Vec<DateTime<Utc>> = storage.get_posts_create_dates()?;
    drop(storage);

    let years = group_to_years(&dates);

    let tags = aggregated_tags
        .iter()
        .map(|tag| {
            let ix = (tag.count as f32 / total_posts as f32 * 10.0) as usize;
            Tag {
                title: tag.title.clone(),
                level: ix,
            }
        })
        .collect();

    Ok(Archive { tags, years })
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
                    month: month as i32,
                    posts: mg.count() as i32,
                })
                .fold(Year::new(y), |mut y, m| {
                    y.append_month(m);
                    y
                })
        })
        .collect()
}

pub fn get_small_posts(
    storage: MutexGuard<Sqlite>,
    page_size: i32,
    request: Option<PostsRequest>,
) -> Result<ApiResult<SmallPost>> {
    let request = request.unwrap_or_default();
    let page = request.page.unwrap_or(1);

    let total_posts_count = storage.count_posts(request.clone())?;
    let pages_count = ceil_div(total_posts_count, page_size);

    let posts = storage.get_small_posts(page_size, page_size * (page - 1), request)?;

    Ok(ApiResult {
        result: update_short_text(posts),
        pages: pages_count,
        page,
        count: total_posts_count,
        status: "success",
    })
}

pub fn get_posts(
    storage: &MutexGuard<Sqlite>,
    page_size: i32,
    request: PostsRequest,
) -> Result<ApiResult<Post>> {
    let page = request.page.unwrap_or(1);

    let mut req = request;
    req.include_private = Some(true);
    let total_posts_count = storage.count_posts(req)?;
    let pages_count = ceil_div(total_posts_count, page_size);

    let posts = storage.get_posts(page_size, page_size * (page - 1))?;

    Ok(ApiResult {
        result: posts,
        pages: pages_count,
        page,
        count: total_posts_count,
        status: "success",
    })
}

/// Integer division with upper rounding (ceil)
fn ceil_div(dividend: i32, divider: i32) -> i32 {
    dividend / divider + (dividend % divider).signum()
}

fn update_short_text(mut posts: Vec<SmallPost>) -> Vec<SmallPost> {
    for post in &mut posts {
        if post.markdown
            && let Ok(text) = markdown2html(&post.short_text)
        {
            post.short_text = text;
        }
    }
    posts
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_in_result)]
    #![allow(clippy::unwrap_used)]
    use super::*;
    use chrono::NaiveDate;
    use rstest::rstest;

    #[rstest]
    #[case(0, 0)]
    #[case(20, 1)]
    #[case(21, 2)]
    #[case(40, 2)]
    #[case(41, 3)]
    #[case(60, 3)]
    #[case(66, 4)]
    fn ceil_div_tests(#[case] dividend: i32, #[case] expected: i32) {
        // arrange
        let page_size = 20;

        // act
        let actual = ceil_div(dividend, page_size);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn group_to_years_tests() {
        // arrange
        let dt1 = NaiveDate::from_ymd_opt(2015, 2, 2)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();

        let dt2 = NaiveDate::from_ymd_opt(2015, 2, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();

        let dt3 = NaiveDate::from_ymd_opt(2015, 3, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();

        let dt4 = NaiveDate::from_ymd_opt(2016, 3, 3)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .latest()
            .unwrap();

        let dates = vec![dt1, dt2, dt3, dt4];

        // act
        let actual = group_to_years(&dates);

        // assert
        assert_eq!(2, actual.len());
        assert_eq!(2015, actual[0].year);
        assert_eq!(3, actual[0].posts);
        assert_eq!(2, actual[0].months.len());
        assert_eq!(2016, actual[1].year);
        assert_eq!(1, actual[1].posts);
        assert_eq!(1, actual[1].months.len());
    }
}
