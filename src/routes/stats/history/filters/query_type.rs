// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Type Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams};

/// Only show queries with the specified query type
pub fn filter_query_type<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(query_type) = params.query_type {
        Box::new(queries_iter.filter(move |query| query.query_type == query_type))
    } else {
        queries_iter
    }
}

#[cfg(test)]
mod test {
    use super::filter_query_type;
    use crate::{
        ftl::{FtlQuery, FtlQueryType},
        routes::stats::history::{endpoints::HistoryParams, testing::test_queries}
    };

    /// Only return queries with the specified query type
    #[test]
    fn test_filter_query_type() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[3], &queries[6], &queries[8]];
        let filtered_queries: Vec<&FtlQuery> = filter_query_type(
            Box::new(queries.iter()),
            &HistoryParams {
                query_type: Some(FtlQueryType::A),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
