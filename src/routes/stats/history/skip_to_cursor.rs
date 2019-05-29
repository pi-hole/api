// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Skip To Cursor Functionality
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries, ftl::FtlQuery, routes::stats::history::endpoints::HistoryParams
};
use diesel::{prelude::*, sqlite::Sqlite};

/// Skip iteration until the query which corresponds to the cursor.
pub fn skip_to_cursor<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams
) -> Box<dyn Iterator<Item = &'a FtlQuery> + 'a> {
    if let Some(cursor) = params.cursor {
        if let Some(id) = cursor.id {
            Box::new(queries_iter.skip_while(move |query| query.id as i32 != id))
        } else if let Some(db_id) = cursor.db_id {
            Box::new(queries_iter.skip_while(move |query| query.database_id != db_id))
        } else {
            // No cursor data, don't skip any queries
            queries_iter
        }
    } else {
        queries_iter
    }
}

/// Skip database queries until the query which corresponds to the cursor.
pub fn skip_to_cursor_db(
    db_query: queries::BoxedQuery<Sqlite>,
    start_id: Option<i64>
) -> queries::BoxedQuery<Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    // If a start ID is given, ignore any queries before it
    if let Some(start_id) = start_id {
        db_query.filter(id.le(start_id as i32))
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{skip_to_cursor, skip_to_cursor_db};
    use crate::{
        databases::ftl::{connect_to_ftl_test_db, FtlDbQuery},
        ftl::FtlQuery,
        routes::stats::history::{
            database::execute_query,
            endpoints::{HistoryCursor, HistoryParams},
            testing::test_queries
        }
    };
    use diesel::prelude::*;

    /// Skip queries according to the cursor (dnsmasq ID)
    #[test]
    fn dnsmasq_cursor() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(7).collect();
        let filtered_queries: Vec<&FtlQuery> = skip_to_cursor(
            Box::new(queries.iter()),
            &HistoryParams {
                cursor: Some(HistoryCursor {
                    id: Some(8),
                    db_id: None
                }),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Skip queries according to the cursor (database ID)
    #[test]
    fn database_cursor() {
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().skip(4).collect();
        let filtered_queries: Vec<&FtlQuery> = skip_to_cursor(
            Box::new(queries.iter()),
            &HistoryParams {
                cursor: Some(HistoryCursor {
                    id: None,
                    db_id: Some(99)
                }),
                ..HistoryParams::default()
            }
        )
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Search starts from the start_id. This is a database filter.
    #[test]
    fn database() {
        use crate::databases::ftl::queries::dsl::*;

        let expected_queries = vec![FtlDbQuery {
            id: 1,
            timestamp: 0,
            query_type: 6,
            status: 3,
            domain: "1.1.1.10.in-addr.arpa".to_owned(),
            client: "127.0.0.1".to_owned(),
            upstream: None
        }];

        let db_query = skip_to_cursor_db(queries.into_boxed(), Some(1));
        let filtered_queries = execute_query(&connect_to_ftl_test_db(), db_query).unwrap();

        assert_eq!(filtered_queries, expected_queries);
    }
}
