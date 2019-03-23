// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Blocked Query Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries,
    ftl::{FtlQuery, BLOCKED_STATUSES},
    routes::stats::history::endpoints::HistoryParams
};
use diesel::{prelude::*, sqlite::Sqlite};

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

/// Only show allowed/blocked queries in database results
pub fn filter_blocked_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(blocked) = params.blocked {
        if blocked {
            db_query.filter(status.eq_any(&BLOCKED_STATUSES))
        } else {
            db_query.filter(status.ne_all(&BLOCKED_STATUSES))
        }
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{filter_blocked, filter_blocked_db};
    use crate::{
        databases::ftl::connect_to_test_db,
        ftl::{FtlQuery, BLOCKED_STATUSES},
        routes::stats::history::{
            database::execute_query, endpoints::HistoryParams, testing::test_queries
        }
    };
    use diesel::prelude::*;

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

    /// Only queries with a blocked/unblocked status are returned. This is a
    /// database filter.
    #[test]
    fn database() {
        use crate::databases::ftl::queries::dsl::*;

        let blocked = false;
        let params = HistoryParams {
            blocked: Some(blocked),
            ..HistoryParams::default()
        };

        let db_query = filter_blocked_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert!(!BLOCKED_STATUSES.contains(&(query.status as i32)));
        }
    }
}
