// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Time Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Filter out queries before the `from` timestamp
pub fn filter_time_from<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(from) = params.from {
        Box::new(queries_iter.filter(move |query| query.timestamp as u64 >= from))
    } else {
        queries_iter
    }
}

/// Filter out queries after the `until` timestamp
pub fn filter_time_until<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(until) = params.until {
        Box::new(queries_iter.filter(move |query| query.timestamp as u64 <= until))
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::{filter_time_from, filter_time_until};
    use crate::{
        ftl::FtlQuery,
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Skip queries before the timestamp
    #[test]
    fn test_filter_time_from() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_from(
            Box::new(queries.iter()),
            &HistoryParams {
                from: Some(4),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries after the timestamp
    #[test]
    fn test_filter_time_until() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().take(5).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_until(
            Box::new(queries.iter()),
            &HistoryParams {
                until: Some(4),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
