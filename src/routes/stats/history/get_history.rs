// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
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
    env::Env,
    ftl::{FtlMemory, FtlQuery},
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Reply}
};
use rocket_contrib::json::JsonValue;

/// Get the query history according to the specified parameters
pub fn get_history(ftl_memory: &FtlMemory, env: &Env, params: HistoryParams) -> Reply {
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

    // Get the limit
    let limit = params.limit.unwrap_or(100);

    // Apply the limit (plus one to get the cursor) and collect the queries
    let history: Vec<&FtlQuery> = queries_iter.take(limit + 1).collect();

    // Get the next cursor from the the "limit+1"-th query, which is the query
    // at index "limit".
    // If no such query exists, the cursor will be None (null in JSON).
    // The cursor is a JSON object with either the DB ID of the query if it is
    // non-zero, or the normal ID. Example: { id: 1, db_id: null }
    let next_cursor = history.get(limit).map(|query: &&FtlQuery| {
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

    // Map the queries into the output format
    let history: Vec<JsonValue> = history
        .into_iter()
        // Only take up to the limit this time, not including the last query,
        // because it was just used to get the cursor
        .take(limit)
        .map(map_query_to_json(ftl_memory, &lock)?)
        .collect();

    reply_data(json!({
        "cursor": next_cursor,
        "history": history
    }))
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
            .expect_json(json!({
                "history": [],
                "cursor": None::<()>
            }))
            .test();
    }
}
