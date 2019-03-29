// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Main History Function
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use super::{
    endpoints::{HistoryCursor, HistoryParams},
    filters::*,
    map_query_to_json::map_query_to_json,
    skip_to_cursor::skip_to_cursor
};
use crate::{
    databases::ftl::FtlDatabase,
    env::Env,
    ftl::{FtlMemory, FtlQuery},
    routes::stats::history::database::load_queries_from_database,
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Reply}
};
use diesel::sqlite::SqliteConnection;
use rocket_contrib::json::JsonValue;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the query history according to the specified parameters
pub fn get_history(
    ftl_memory: &FtlMemory,
    env: &Env,
    params: HistoryParams,
    db: &FtlDatabase
) -> Reply {
    // Check if query details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(env)? >= FtlPrivacyLevel::Maximum {
        // `None::<()>` represents `null` in JSON. It needs the type parameter because
        // it doesn't know what type of Option it is (`Option<T>`)
        return reply_data(json!({
            "cursor": None::<()>,
            "history": []
        }));
    }

    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;
    let queries = ftl_memory.queries(&lock)?;

    // The following code uses a boxed iterator,
    // Box<dyn Iterator<Item = &FtlQuery>>
    //
    // When you make an iterator chain, it modifies the type of the iterator.
    // Ex. slice.iter().filter(..).map(..) might look like Map<Filter<Iter<T>>, I>
    //
    // Because of this, if you want to dynamically create an iterator like we do
    // below, the iterator must be kept on the heap instead of the stack
    // because the type of the iterator is not known at compile time.
    //
    // What we do know for certain about the iterator is that it implements
    // Iterator<Item = &FtlQuery>, so using Box we can dynamically add many
    // combinations of modifiers to the iterator and not worry about the real
    // type.

    // Start making an iterator by getting valid query references (FTL allocates
    // more than it uses).
    let queries_iter = Box::new(
        queries
            .iter()
            // Get the most recent queries first
            .rev()
            // Skip the uninitialized queries
            .skip(queries.len() - counters.total_queries as usize)
    );

    // If there is a cursor, skip to the referenced query
    let queries_iter = skip_to_cursor(queries_iter, &params);

    // Apply filters
    let queries_iter = filter_private_queries(queries_iter);
    let queries_iter = filter_setup_vars_setting(queries_iter, env)?;
    let queries_iter = filter_time_from(queries_iter, &params);
    let queries_iter = filter_time_until(queries_iter, &params);
    let queries_iter = filter_query_type(queries_iter, &params);
    let queries_iter = filter_upstream(queries_iter, &params, ftl_memory, &lock)?;
    let queries_iter = filter_domain(queries_iter, &params, ftl_memory, &lock)?;
    let queries_iter = filter_client(queries_iter, &params, ftl_memory, &lock)?;
    let queries_iter = filter_status(queries_iter, &params);
    let queries_iter = filter_blocked(queries_iter, &params);
    let queries_iter = filter_dnssec(queries_iter, &params);
    let queries_iter = filter_reply(queries_iter, &params);
    let queries_iter = filter_excluded_domains(queries_iter, env, ftl_memory, &lock)?;
    let queries_iter = filter_excluded_clients(queries_iter, env, ftl_memory, &lock)?;

    // Get the limit
    let limit = params.limit.unwrap_or(100);

    // Apply the limit (plus one to get the cursor) and collect the queries
    let history: Vec<&FtlQuery> = queries_iter.take(limit + 1).collect();

    // Get the next cursor from the the "limit+1"-th query, which is the query
    // at index "limit".
    // If no such query exists, the cursor will be None (null in JSON).
    // The cursor is a JSON object with either the DB ID of the query if it is
    // non-zero, or the normal ID. Example: { id: 1, db_id: null }
    let mut next_cursor = history.get(limit).map(|query: &&FtlQuery| {
        let db_id = if query.database_id != 0 {
            Some(query.database_id)
        } else {
            None
        };
        let id = if db_id.is_none() {
            Some(query.id)
        } else {
            None
        };

        let cursor = HistoryCursor { id, db_id };

        cursor.as_base64().unwrap()
    });

    // Get the last database ID of the in-memory queries we found, or if we
    // didn't find any in-memory queries, get the database ID in the cursor.
    // This is done in case we have to query the database to get more queries.
    // If no ID is found, then the search will start with the most recent
    // queries in the database.
    let last_db_id = history
        .last()
        // Subtract one from the database ID so that the database search starts
        // with the next query instead of the last one we found
        .map(|query| query.database_id - 1)
        // If no queries were found, then use the cursor's database ID
        .or_else(|| params.cursor.map(|cursor| cursor.db_id).unwrap_or(None));

    // Map the queries into the output format
    let history: Vec<JsonValue> = history
            .into_iter()
            // Only take up to the limit this time, not including the last query,
            // because it was just used to get the cursor
            .take(limit)
            .map(map_query_to_json(ftl_memory, &lock)?)
            .collect();

    // If there are not enough queries to reach the limit (next cursor is null),
    // there is a specified timestamp, and the timespan is not entirely within
    // the last 24 hours, then search the database for more queries.
    let history = if next_cursor.is_none()
        && (params.from.is_some() || params.until.is_some())
        && !is_within_24_hours(params.from, params.until)
    {
        // Load queries from the database
        let (db_queries, cursor) =
            load_queries_from_database(db as &SqliteConnection, last_db_id, &params, env, limit)?;

        // Map the queries into JSON
        let db_queries = db_queries.into_iter().map(Into::into);

        // Update the cursor
        next_cursor = cursor.map(|cursor| cursor.as_base64().unwrap());

        // Extend history with the database queries
        history.into_iter().chain(db_queries).collect()
    } else {
        history
    };

    reply_data(json!({
        "cursor": next_cursor,
        "history": history
    }))
}

/// Check if the timespan is completely within the last 24 hours
fn is_within_24_hours(from: Option<u64>, until: Option<u64>) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is older than epoch")
        .as_secs();
    let yesterday = now - 60 * 60 * 24;

    match (from, until) {
        (Some(from), Some(until)) => until >= from && from > yesterday,
        (Some(from), None) => from > yesterday,
        (None, Some(_)) => {
            // With an unbounded from, any query older than the until time could
            // be used
            false
        }
        (None, None) => {
            // No bounds on the query time
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::ShmLockGuard,
        routes::stats::history::{
            map_query_to_json::map_query_to_json,
            testing::{test_memory, test_queries}
        },
        testing::TestBuilder
    };
    use rocket_contrib::json::JsonValue;

    /// The default behavior lists the first 100 non-private queries sorted by
    /// most recent
    #[test]
    fn default_params() {
        let ftl_memory = test_memory();
        let mut expected_queries = test_queries();

        // The private query should be ignored
        expected_queries.remove(8);

        let history: Vec<JsonValue> = expected_queries
            .iter()
            .rev()
            .map(map_query_to_json(&ftl_memory, &ShmLockGuard::Test).unwrap())
            .collect();

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .ftl_memory(ftl_memory)
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .need_database(true)
            .expect_json(json!({
                "history": history,
                "cursor": None::<()>
            }))
            .test();
    }

    /// When the limit is specified, only that many queries will be shown
    #[test]
    fn limit() {
        let ftl_memory = test_memory();
        let mut expected_queries = test_queries();

        // The private query should be ignored
        expected_queries.remove(8);

        let history: Vec<JsonValue> = expected_queries
            .iter()
            .rev()
            .take(5)
            .map(map_query_to_json(&ftl_memory, &ShmLockGuard::Test).unwrap())
            .collect();

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?limit=5")
            .ftl_memory(ftl_memory)
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .need_database(true)
            .expect_json(json!({
                "history": history,
                "cursor": "eyJpZCI6bnVsbCwiZGJfaWQiOjk3fQ=="
            }))
            .test();
    }

    /// Maximum privacy shows no queries
    #[test]
    fn privacy_max() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=3")
            .ftl_memory(test_memory())
            .need_database(true)
            .expect_json(json!({
                "history": [],
                "cursor": None::<()>
            }))
            .test();
    }

    /// Load queries from the database
    #[test]
    fn database() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/history?from=177180&until=177181")
            .ftl_memory(test_memory())
            .file(PiholeFile::SetupVars, "")
            .file(PiholeFile::FtlConfig, "")
            .need_database(true)
            .expect_json(json!({
                "history": [
                    {
                        "timestamp": 177_180,
                        "type": 6,
                        "status": 2,
                        "domain": "4.4.8.8.in-addr.arpa",
                        "client": "127.0.0.1",
                        "dnssec": 5,
                        "reply": 0,
                        "response_time": 0
                    },
                    {
                        "timestamp": 177_180,
                        "type": 6,
                        "status": 3,
                        "domain": "1.1.1.10.in-addr.arpa",
                        "client": "127.0.0.1",
                        "dnssec": 5,
                        "reply": 0,
                        "response_time": 0
                    }
                ],
                "cursor": None::<()>
            }))
            .test();
    }
}
