// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Blocked Query Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Only show allowed/blocked queries
pub fn filter_blocked<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(blocked) = params.blocked {
        if blocked {
            Box::new(queries_iter.filter(|query| query.is_blocked()))
        } else {
            Box::new(queries_iter.filter(|query| !query.is_blocked()))
        }
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::filter_blocked;
    use crate::{
        ftl::FtlQuery,
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Only return allowed/blocked queries
    #[test]
    fn test_filter_blocked() {
        let queries = test_queries();
        let expected_queries = vec![&queries[3], &queries[5], &queries[6], &queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_blocked(
            Box::new(queries.iter()),
            &HistoryParams {
                blocked: Some(true),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
