// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Over Time Database Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    ftl::BLOCKED_STATUSES,
    routes::{auth::User, stats::over_time_history::OverTimeItem},
    util::{reply_result, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt};
use failure::ResultExt;
use std::collections::HashMap;

/// Get the query history over time from the database
/// (separated into blocked and not blocked)
#[get("/stats/database/overTime/history?<from>&<until>&<interval>")]
pub fn over_time_history_db(
    from: u64,
    until: u64,
    interval: Option<usize>,
    _auth: User,
    db: FtlDatabase
) -> Reply {
    reply_result(over_time_history_db_impl(
        from,
        until,
        interval.unwrap_or(600),
        &db as &SqliteConnection
    ))
}

/// Get the over time data from the database
fn over_time_history_db_impl(
    from: u64,
    until: u64,
    interval: usize,
    db: &SqliteConnection
) -> Result<Vec<OverTimeItem>, Error> {
    let (from, until) = align_from_until(from, until, interval as u64)?;

    // Get the overTime data
    let total_intervals = get_total_intervals(from, until, interval, db)?;
    let blocked_intervals = get_blocked_intervals(from, until, interval, db)?;

    let mut over_time: Vec<OverTimeItem> = Vec::with_capacity((until - from) as usize / interval);

    // For each interval's timestamp, create the overTime slot
    for timestamp in (from..until).step_by(interval) {
        let timestamp_key = &(timestamp as i32);
        let total_queries = *total_intervals.get(timestamp_key).unwrap_or(&0) as usize;
        let blocked_queries = *blocked_intervals.get(timestamp_key).unwrap_or(&0) as usize;

        over_time.push(OverTimeItem {
            // Display the timestamps as centered in the overTime slot interval
            timestamp: timestamp + (interval / 2) as u64,
            total_queries,
            blocked_queries
        });
    }

    Ok(over_time)
}

/// Align `from` and `until` with the interval. Also check that the time
/// interval is increasing from `from` to `until`. If it is not, an error is
/// returned.
pub fn align_from_until(from: u64, until: u64, interval: u64) -> Result<(u64, u64), Error> {
    let is_range_increasing = from < until;

    if !is_range_increasing {
        // The timestamps should increase from "from" to "until"
        return Err(Error::from(ErrorKind::BadRequest));
    }

    // Align timestamps with the interval
    let from = from - (from % interval);
    let until = until - (until % interval) + interval;

    Ok((from, until))
}

/// Get the over time data for all queries from the database
fn get_total_intervals(
    from: u64,
    until: u64,
    interval: usize,
    db: &SqliteConnection
) -> Result<HashMap<i32, i64>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // SQL snippet for calculating the interval timestamp of the query
    let interval_sql = sql(&format!(
        "(timestamp / {interval}) * {interval}",
        interval = interval
    ));

    // Create SQL query
    let sql_query = queries
        .select((&interval_sql, sql::<BigInt>("COUNT(*)")))
        .filter(status.ne(0))
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.lt(until as i32))
        .group_by(&interval_sql);

    // Execute SQL query
    Ok(sql_query
        .load(&db as &SqliteConnection)
        .context(ErrorKind::FtlDatabase)?
        // Convert to HashMap
        .into_iter()
        .collect())
}

/// Get the over time data for blocked queries from the database
fn get_blocked_intervals(
    from: u64,
    until: u64,
    interval: usize,
    db: &SqliteConnection
) -> Result<HashMap<i32, i64>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // SQL snippet for calculating the interval timestamp of the query
    let interval_sql = sql(&format!(
        "(timestamp / {interval}) * {interval}",
        interval = interval
    ));

    // Create SQL query
    let sql_query = queries
        .select((&interval_sql, sql::<BigInt>("COUNT(*)")))
        .filter(status.eq_any(&BLOCKED_STATUSES))
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.lt(until as i32))
        .group_by(&interval_sql);

    // Execute SQL query
    Ok(sql_query
        .load(&db as &SqliteConnection)
        .context(ErrorKind::FtlDatabase)?
        // Convert to HashMap
        .into_iter()
        .collect())
}

#[cfg(test)]
mod test {
    use super::{get_blocked_intervals, get_total_intervals, over_time_history_db_impl};
    use crate::{
        databases::ftl::connect_to_ftl_test_db, routes::stats::over_time_history::OverTimeItem
    };
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 164_400;
    const UNTIL_TIMESTAMP: u64 = 177_000;
    const INTERVAL: usize = 600;

    /// Verify the over time data is retrieved correctly
    #[test]
    fn over_time_history_impl() {
        let expected = vec![
            OverTimeItem {
                timestamp: 164_700,
                total_queries: 26,
                blocked_queries: 0
            },
            OverTimeItem {
                timestamp: 165_300,
                total_queries: 7,
                blocked_queries: 0
            },
            OverTimeItem {
                timestamp: 165_900,
                total_queries: 0,
                blocked_queries: 0
            },
        ];

        let db = connect_to_ftl_test_db();
        let actual = over_time_history_db_impl(164_400, 165_600, INTERVAL, &db).unwrap();

        assert_eq!(actual, expected);
    }

    /// Verify the total intervals are retrieved correctly
    #[test]
    fn total_intervals() {
        let mut expected = HashMap::new();
        expected.insert(164_400, 26);
        expected.insert(165_000, 7);
        expected.insert(168_600, 3);
        expected.insert(172_200, 3);
        expected.insert(174_000, 8);
        expected.insert(175_800, 3);

        let db = connect_to_ftl_test_db();
        let actual = get_total_intervals(FROM_TIMESTAMP, UNTIL_TIMESTAMP, INTERVAL, &db).unwrap();

        assert_eq!(actual, expected);
    }

    /// Verify the blocked intervals are retrieved correctly (there are none)
    #[test]
    fn blocked_intervals() {
        let expected = HashMap::new();

        let db = connect_to_ftl_test_db();
        let actual = get_blocked_intervals(FROM_TIMESTAMP, UNTIL_TIMESTAMP, INTERVAL, &db).unwrap();

        assert_eq!(actual, expected);
    }
}
