// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Type Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries, ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams
};
use diesel::{prelude::*, sqlite::Sqlite};

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

/// Only show queries with the specified query type in database results
pub fn filter_query_type_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(search_query_type) = params.query_type {
        db_query.filter(query_type.eq(search_query_type as i32))
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{filter_query_type, filter_query_type_db};
    use crate::{
        databases::ftl::connect_to_test_db,
        ftl::{FtlQuery, FtlQueryType},
        routes::stats::history::{
            database::execute_query, endpoints::HistoryParams, testing::test_queries
        }
    };
    use diesel::prelude::*;

    /// Only return queries with the specified query type
    #[test]
    fn query_type() {
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

    /// Only queries with the input query type are returned. This is a database
    /// filter.
    #[test]
    fn database() {
        use crate::databases::ftl::queries::dsl::*;

        let expected_query_type = FtlQueryType::PTR;
        let params = HistoryParams {
            query_type: Some(expected_query_type),
            ..HistoryParams::default()
        };

        let db_query = filter_query_type_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert_eq!(query.query_type, expected_query_type as i32);
        }
    }
}
