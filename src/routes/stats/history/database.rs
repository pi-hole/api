// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Database Integration For History
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDbQuery,
    ftl::FtlQueryStatus,
    routes::stats::history::endpoints::{HistoryCursor, HistoryParams},
    util::{Error, ErrorKind}
};
use diesel::{prelude::*, query_builder::BoxedSelectStatement};
use failure::ResultExt;

/// Load queries from the database according to the parameters. A cursor is
/// also returned, if more queries can be loaded.
///
/// # Arguments:
/// - `db`: A connection to the FTL database
/// - `start_id`: The query ID to start searching from. If this is `None` then
///   the search will start from the most recent queries
/// - `params`: Parameters given to the history endpoint (filters)
/// - `limit`: The maximum number of queries to load
pub fn load_queries_from_database(
    db: &SqliteConnection,
    start_id: Option<i64>,
    params: &HistoryParams,
    limit: usize
) -> Result<(Vec<FtlDbQuery>, Option<HistoryCursor>), Error> {
    // Use the Diesel DSL of this table for easy querying
    use crate::databases::ftl::queries::dsl::*;

    // Start creating the database query
    let mut db_query: BoxedSelectStatement<_, _, _> = queries
        // The query must be boxed, because we are dynamically building it
        .into_boxed()
        // Take up to the limit, plus one to build the cursor
        .limit((limit + 1) as i64)
        // Start with the most recently inserted queries
        .order(id.desc());

    // If a start ID is given, ignore any queries before it
    if let Some(start_id) = start_id {
        db_query = db_query.filter(id.le(start_id as i32));
    }

    // Apply filters

    if let Some(from) = params.from {
        db_query = db_query.filter(timestamp.ge(from as i32));
    }

    if let Some(until) = params.until {
        db_query = db_query.filter(timestamp.le(until as i32));
    }

    if let Some(ref search_domain) = params.domain {
        db_query = db_query.filter(domain.like(format!("%{}%", search_domain)));
    }

    if let Some(ref search_client) = params.client {
        db_query = db_query.filter(client.like(format!("%{}%", search_client)));
    }

    if let Some(ref search_upstream) = params.upstream {
        db_query = db_query.filter(upstream.like(format!("%{}%", search_upstream)));
    }

    if let Some(search_query_type) = params.query_type {
        db_query = db_query.filter(query_type.eq(search_query_type as i32));
    }

    if let Some(search_status) = params.status {
        db_query = db_query.filter(status.eq(search_status as i32));
    }

    // A list of query statuses which mark a query as blocked
    let blocked_statuses = [
        FtlQueryStatus::Gravity as i32,
        FtlQueryStatus::Wildcard as i32,
        FtlQueryStatus::Blacklist as i32,
        FtlQueryStatus::ExternalBlock as i32
    ];

    if let Some(blocked) = params.blocked {
        db_query = if blocked {
            db_query.filter(status.eq_any(&blocked_statuses))
        } else {
            db_query.filter(status.ne_all(&blocked_statuses))
        };
    }

    // Execute the query and load the results
    let mut results: Vec<FtlDbQuery> = db_query
        .load(db as &SqliteConnection)
        .context(ErrorKind::FtlDatabase)
        .map_err(Error::from)?;

    // If more queries could be loaded beyond the given limit (if we loaded
    // limit + 1 queries), then set the cursor to use the limit + 1 query's ID.
    let cursor = if results.len() == limit + 1 {
        Some(HistoryCursor {
            id: None,
            db_id: Some(results[limit].id.unwrap() as i64)
        })
    } else {
        None
    };

    // Drop the limit + 1 query, if it exists. It is only used to determine
    // the new cursor.
    results.truncate(limit);

    Ok((results, cursor))
}

#[cfg(test)]
mod test {
    use super::load_queries_from_database;
    use crate::{
        databases::{ftl::FtlDbQuery, TEST_DATABASE_PATH},
        ftl::{FtlQueryStatus, FtlQueryType},
        routes::stats::history::endpoints::{HistoryCursor, HistoryParams}
    };
    use diesel::prelude::*;

    /// Connect to the testing database
    fn connect_to_test_db() -> SqliteConnection {
        SqliteConnection::establish(TEST_DATABASE_PATH).unwrap()
    }

    /// Search starts from the start_id
    #[test]
    fn start_id() {
        let expected_queries = vec![FtlDbQuery {
            id: Some(1),
            timestamp: 0,
            query_type: 6,
            status: 3,
            domain: "1.1.1.10.in-addr.arpa".to_owned(),
            client: "127.0.0.1".to_owned(),
            upstream: None
        }];

        let (queries, cursor) = load_queries_from_database(
            &connect_to_test_db(),
            Some(1),
            &HistoryParams::default(),
            100
        )
        .unwrap();

        assert_eq!(queries, expected_queries);
        assert_eq!(cursor, None);
    }

    /// Queries are ordered by id, descending
    #[test]
    fn order_by_id() {
        let (queries, cursor) = load_queries_from_database(
            &connect_to_test_db(),
            Some(2),
            &HistoryParams::default(),
            100
        )
        .unwrap();

        assert_eq!(cursor, None);
        assert_eq!(queries.len(), 2);
        assert!(queries[0].id > queries[1].id);
    }

    /// The max number of queries returned is specified by the limit
    #[test]
    fn limit() {
        let expected_cursor = Some(HistoryCursor {
            id: None,
            db_id: Some(1)
        });

        let (queries, cursor) = load_queries_from_database(
            &connect_to_test_db(),
            Some(3),
            &HistoryParams::default(),
            2
        )
        .unwrap();

        assert_eq!(queries.len(), 2);
        assert_eq!(cursor, expected_cursor);
    }

    /// Only queries newer than `from` are returned
    #[test]
    fn from() {
        let from = 177_179;
        let params = HistoryParams {
            from: Some(from),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert!(queries[0].timestamp >= from as i32);
    }

    /// Only queries older than `until` are returned
    #[test]
    fn until() {
        let until = 1;
        let params = HistoryParams {
            until: Some(until),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert!(queries[0].timestamp <= until as i32);
    }

    /// Only queries with domains similar to the input are returned.
    #[test]
    fn domain() {
        let params = HistoryParams {
            domain: Some("goog".to_owned()),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].domain, "google.com");
    }

    /// Only queries with a client similar to the input are returned.
    #[test]
    fn client() {
        let params = HistoryParams {
            client: Some("10.1".to_owned()),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].client, "10.1.1.1");
    }

    /// Only queries with an upstream similar to the input are returned.
    #[test]
    fn upstream() {
        let params = HistoryParams {
            upstream: Some("8.8.8".to_owned()),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].upstream, Some("8.8.8.8".to_owned()));
    }

    /// Only queries with the input query type are returned.
    #[test]
    fn query_type() {
        let query_type = FtlQueryType::PTR;
        let params = HistoryParams {
            query_type: Some(query_type),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].query_type, query_type as i32);
    }

    /// Only queries with the input query status are returned.
    #[test]
    fn status() {
        let status = FtlQueryStatus::Forward;
        let params = HistoryParams {
            status: Some(status),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].status, status as i32);
    }

    /// Only queries with a blocked/unblocked status are returned.
    #[test]
    fn blocked() {
        let blocked = false;
        let params = HistoryParams {
            blocked: Some(blocked),
            ..HistoryParams::default()
        };

        let (queries, _) =
            load_queries_from_database(&connect_to_test_db(), None, &params, 1).unwrap();

        assert_eq!(queries.len(), 1);
        assert!(
            match FtlQueryStatus::from_number(queries[0].status as isize).unwrap() {
                FtlQueryStatus::Gravity
                | FtlQueryStatus::Wildcard
                | FtlQueryStatus::Blacklist
                | FtlQueryStatus::ExternalBlock => blocked,
                _ => !blocked
            }
        );
    }
}
