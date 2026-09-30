/// Returns the 1-based page number and the row offset for a requested page.
/// Missing, zero or negative pages are treated as the first page.
#[must_use]
pub fn page_offset(page: Option<i32>, page_size: i32) -> (i32, i32) {
    let page = page.unwrap_or(1).max(1);
    (page, page_size.saturating_mul(page - 1))
}

/// Number of pages needed to show `count` items, `page_size` items per page.
#[must_use]
pub fn pages_count(count: i32, page_size: i32) -> i32 {
    count / page_size + (count % page_size).signum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(None, 1, 0)]
    #[case(Some(1), 1, 0)]
    #[case(Some(3), 3, 40)]
    #[case(Some(0), 1, 0)]
    #[case(Some(-5), 1, 0)]
    #[case(Some(i32::MIN), 1, 0)]
    #[case(Some(i32::MAX), i32::MAX, i32::MAX)]
    fn page_offset_tests(
        #[case] page: Option<i32>,
        #[case] expected_page: i32,
        #[case] expected_offset: i32,
    ) {
        // arrange
        let page_size = 20;

        // act
        let actual = page_offset(page, page_size);

        // assert
        assert_eq!((expected_page, expected_offset), actual);
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 1)]
    #[case(20, 1)]
    #[case(21, 2)]
    #[case(40, 2)]
    #[case(41, 3)]
    #[case(60, 3)]
    #[case(66, 4)]
    fn pages_count_tests(#[case] count: i32, #[case] expected: i32) {
        // arrange
        let page_size = 20;

        // act
        let actual = pages_count(count, page_size);

        // assert
        assert_eq!(expected, actual);
    }
}
