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
    databases::ftl::{queries, FtlDbQuery},
    env::Env,
    routes::stats::history::{
        endpoints::{HistoryCursor, HistoryParams},
        filters::*,
        skip_to_cursor::skip_to_cursor_db
    },
    util::{Error, ErrorKind}
};
use diesel::{prelude::*, sqlite::Sqlite};
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
    env: &Env,
    limit: usize
) -> Result<(Vec<FtlDbQuery>, Option<HistoryCursor>), Error> {
    // Use the Diesel DSL of this table for easy querying
    use crate::databases::ftl::queries::dsl::*;

    // Start creating the database query
    let db_query = queries
        // The query must be boxed, because we are dynamically building it
        .into_boxed()
        // Take up to the limit, plus one to build the cursor
        .limit((limit + 1) as i64)
        // Start with the most recently inserted queries
        .order(id.desc());

    // If a start ID is given, ignore any queries before it
    let db_query = skip_to_cursor_db(db_query, start_id);

    // Apply filters
    let db_query = filter_time_from_db(db_query, params);
    let db_query = filter_time_until_db(db_query, params);
    let db_query = filter_domain_db(db_query, params);
    let db_query = filter_client_db(db_query, params);
    let db_query = filter_upstream_db(db_query, params);
    let db_query = filter_query_type_db(db_query, params);
    let db_query = filter_status_db(db_query, params);
    let db_query = filter_blocked_db(db_query, params);
    let db_query = filter_excluded_domains_db(db_query, env)?;
    let db_query = filter_excluded_clients_db(db_query, env)?;
    let db_query = filter_setup_vars_setting_db(db_query, env)?;

    // Execute the query and load the results
    let mut results: Vec<FtlDbQuery> = execute_query(db, db_query)?;

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

/// Execute a database query for DNS queries on an FTL database.
/// The database could be real, or it could be a test database.
pub fn execute_query(
    db: &SqliteConnection,
    db_query: queries::BoxedQuery<Sqlite>
) -> Result<Vec<FtlDbQuery>, Error> {
    db_query
        .load(db)
        .context(ErrorKind::FtlDatabase)
        .map_err(Error::from)
}

#[cfg(test)]
mod test {
    use super::load_queries_from_database;
    use crate::{
        databases::ftl::connect_to_test_db,
        env::PiholeFile,
        routes::stats::history::endpoints::{HistoryCursor, HistoryParams},
        testing::TestEnvBuilder
    };

    /// Queries are ordered by id, descending
    #[test]
    fn order_by_id() {
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "")
            .build();

        let (queries, cursor) = load_queries_from_database(
            &connect_to_test_db(),
            Some(2),
            &HistoryParams::default(),
            &env,
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
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "")
            .build();
        let expected_cursor = Some(HistoryCursor {
            id: None,
            db_id: Some(1)
        });

        let (queries, cursor) = load_queries_from_database(
            &connect_to_test_db(),
            Some(3),
            &HistoryParams::default(),
            &env,
            2
        )
        .unwrap();

        assert_eq!(queries.len(), 2);
        assert_eq!(cursor, expected_cursor);
    }
}
