// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Time Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries, ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams
};
use diesel::{prelude::*, sqlite::Sqlite};

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

/// Filter out queries before the `from` timestamp in database results
pub fn filter_time_from_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(from) = params.from {
        db_query.filter(timestamp.ge(from as i32))
    } else {
        db_query
    }
}

/// Filter out queries after the `until` timestamp in database results
pub fn filter_time_until_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(until) = params.until {
        db_query.filter(timestamp.le(until as i32))
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{filter_time_from, filter_time_from_db, filter_time_until, filter_time_until_db};
    use crate::{
        databases::ftl::connect_to_ftl_test_db,
        ftl::FtlQuery,
        routes::stats::history::{
            database::execute_query, endpoints::HistoryParams, testing::test_queries
        }
    };
    use diesel::prelude::*;

    /// Skip queries before the timestamp
    #[test]
    fn from() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_from(
            Box::new(queries.iter()),
            &HistoryParams {
                from: Some(263_584),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries after the timestamp
    #[test]
    fn until() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().take(5).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_time_until(
            Box::new(queries.iter()),
            &HistoryParams {
                until: Some(263_584),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only queries newer than `from` are returned. This is a database filter.
    #[test]
    fn from_db() {
        use crate::databases::ftl::queries::dsl::*;

        let from = 177_179;
        let params = HistoryParams {
            from: Some(from),
            ..HistoryParams::default()
        };

        let db_query = filter_time_from_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_ftl_test_db(), db_query).unwrap();

        assert!(filtered_queries
            .iter()
            .all(|query| query.timestamp >= from as i32));
    }

    /// Only queries older than `until` are returned. This is a database filter.
    #[test]
    fn until_db() {
        use crate::databases::ftl::queries::dsl::*;

        let until = 1;
        let params = HistoryParams {
            until: Some(until),
            ..HistoryParams::default()
        };

        let db_query = filter_time_until_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_ftl_test_db(), db_query).unwrap();

        assert!(filtered_queries
            .iter()
            .all(|query| query.timestamp <= until as i32));
    }
}
