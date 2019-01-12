// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Database Integration For History
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::{FtlDatabase, FtlDbQuery},
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
    db: &FtlDatabase,
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

    if let Some(ref upstream) = params.upstream {
        db_query = db_query.filter(forward.like(format!("%{}%", upstream)));
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
