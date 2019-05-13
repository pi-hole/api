// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Status Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries, ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams
};
use diesel::{prelude::*, sqlite::Sqlite};

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

/// Only show queries with the specific status in database results
pub fn filter_status_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(search_status) = params.status {
        db_query.filter(status.eq(search_status as i32))
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{filter_status, filter_status_db};
    use crate::{
        databases::ftl::connect_to_ftl_test_db,
        ftl::{FtlQuery, FtlQueryStatus},
        routes::stats::history::{
            database::execute_query, endpoints::HistoryParams, testing::test_queries
        }
    };
    use diesel::prelude::*;

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

    /// Only queries with the input query status are returned. This is a
    /// database filter.
    #[test]
    fn database() {
        use crate::databases::ftl::queries::dsl::*;

        let expected_status = FtlQueryStatus::Forward;
        let params = HistoryParams {
            status: Some(expected_status),
            ..HistoryParams::default()
        };

        let db_query = filter_status_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_ftl_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert_eq!(query.status, expected_status as i32);
        }
    }
}
