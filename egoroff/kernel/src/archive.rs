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
                level: RANKS[ix],
            }
        })
        .collect();

    Ok(Archive { tags, years })
}

fn group_to_years<'a>(dates: &[DateTime<Utc>]) -> Vec<Year<'a>> {
    dates
        .iter()
        .map(|dt| (dt.year(), dt.month()))
        .group_by(|(year, _month)| *year)
        .into_iter()
        .map(|(y, months)| {
            months
                .group_by(|(_y, m)| *m)
                .into_iter()
                .map(|(month, mg)| Month {
                    month: month as i32,
                    posts: mg.count() as i32,
                    name: MONTHS[month as usize - 1],
                })
                .fold(Year::new(y), |mut y, m| {
                    y.posts += m.posts;
                    y.months.push(m);
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
    let pages_count = count_pages(total_posts_count, page_size);

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
    let pages_count = count_pages(total_posts_count, page_size);

    let posts = storage.get_posts(page_size, page_size * (page - 1))?;

    Ok(ApiResult {
        result: posts,
        pages: pages_count,
        page,
        count: total_posts_count,
        status: "success",
    })
}

fn count_pages(count: i32, page_size: i32) -> i32 {
    count / page_size + i32::from(count % page_size > 0)
}

fn update_short_text(mut posts: Vec<SmallPost>) -> Vec<SmallPost> {
    for mut post in &mut posts {
        if post.markdown {
            if let Ok(text) = markdown2html(&post.short_text) {
                post.short_text = text;
            }
        }
    }
    posts
}

#[cfg(test)]
mod tests {
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
    fn count_pages_tests(#[case] count: i32, #[case] expected: i32) {
        // arrange
        let page_size = 20;

        // act
        let actual = count_pages(count, page_size);

        // assert
        assert_eq!(expected, actual);
    }

    #[test]
    fn group_to_years_tests() {
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
        let actual = group_to_years(&dates);

        // assert
        assert_eq!(2, actual.len());
        assert_eq!(3, actual[0].posts);
        assert_eq!(1, actual[1].posts);
    }
}
