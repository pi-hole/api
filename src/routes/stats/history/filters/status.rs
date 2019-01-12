// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Status Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Only show queries with the specific status
pub fn filter_status<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(status) = params.status {
        Box::new(queries_iter.filter(move |query| query.status == status))
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::filter_status;
    use crate::{
        ftl::{FtlQuery, FtlQueryStatus},
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Only return queries with the specified status
    #[test]
    fn test_filter_status() {
        let queries = test_queries();
        let expected_queries = vec![&queries[3]];
        let filtered_queries: Vec<&FtlQuery> = filter_status(
            Box::new(queries.iter()),
            &HistoryParams {
                status: Some(FtlQueryStatus::Gravity),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
