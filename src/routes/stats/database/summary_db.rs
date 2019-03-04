// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Summary Endpoint - DB Version
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    env::Env,
    ftl::{FtlQueryStatus, FtlQueryType, BLOCKED_STATUSES},
    routes::{
        auth::User,
        stats::{
            database::get_query_type_counts,
            summary::{ReplyTypes, Summary, TotalQueries}
        }
    },
    settings::{ConfigEntry, SetupVarsEntry},
    util::{reply_data, Error, ErrorKind, Reply}
};
use diesel::prelude::*;
use failure::ResultExt;
use rocket::State;

/// Get summary data from database
#[get("/stats/database/summary?<from>&<until>")]
pub fn get_summary_db(
    from: u64,
    until: u64,
    _auth: User,
    db: FtlDatabase,
    env: State<Env>
) -> Reply {
    reply_data(get_summary_impl(
        from,
        until,
        &db as &SqliteConnection,
        &env
    )?)
}

/// Implementation of [`get_summary_db`]
///
/// [`get_summary_db`]: fn.get_summary_db.html
fn get_summary_impl(
    from: u64,
    until: u64,
    db: &SqliteConnection,
    env: &Env
) -> Result<Summary, Error> {
    let query_type_counts = get_query_type_counts(db, from, until)?;

    let total_queries_a = *query_type_counts.get(&FtlQueryType::A).unwrap_or(&0);
    let total_queries_aaaa = *query_type_counts.get(&FtlQueryType::AAAA).unwrap_or(&0);
    let total_queries_any = *query_type_counts.get(&FtlQueryType::ANY).unwrap_or(&0);
    let total_queries_srv = *query_type_counts.get(&FtlQueryType::SRV).unwrap_or(&0);
    let total_queries_soa = *query_type_counts.get(&FtlQueryType::SOA).unwrap_or(&0);
    let total_queries_ptr = *query_type_counts.get(&FtlQueryType::PTR).unwrap_or(&0);
    let total_queries_txt = *query_type_counts.get(&FtlQueryType::TXT).unwrap_or(&0);

    let total_queries = total_queries_a
        + total_queries_aaaa
        + total_queries_any
        + total_queries_srv
        + total_queries_soa
        + total_queries_ptr
        + total_queries_txt;
    let blocked_queries = get_blocked_query_count(db, from, until)?;

    Ok(Summary {
        // Gravity size is set to zero because it is not relevant when looking
        // at long term data
        gravity_size: 0,
        total_queries: TotalQueries {
            A: total_queries_a,
            AAAA: total_queries_aaaa,
            ANY: total_queries_any,
            SRV: total_queries_srv,
            SOA: total_queries_soa,
            PTR: total_queries_ptr,
            TXT: total_queries_txt
        },
        blocked_queries,
        percent_blocked: if total_queries == 0 {
            0f64
        } else {
            (blocked_queries as f64) / (total_queries as f64)
        },
        unique_domains: get_unique_domain_count(db, from, until)?,
        forwarded_queries: get_query_status_count(db, from, until, FtlQueryStatus::Forward)?,
        cached_queries: get_query_status_count(db, from, until, FtlQueryStatus::Cache)?,
        reply_types: ReplyTypes {
            // TODO: use real values when the database supports reply types
            IP: 0,
            CNAME: 0,
            DOMAIN: 0,
            NODATA: 0,
            NXDOMAIN: 0
        },
        // TODO: use real client values when we can accurately determine the number of clients
        total_clients: 0,
        active_clients: 0,
        status: if SetupVarsEntry::BlockingEnabled.is_true(&env)? {
            "enabled"
        } else {
            "disabled"
        }
    })
}

/// Get the number of blocked queries in the specified time range
pub fn get_blocked_query_count(
    db: &SqliteConnection,
    from: u64,
    until: u64
) -> Result<usize, Error> {
    use crate::databases::ftl::queries::dsl::*;

    let count = queries
        .filter(timestamp.le(until as i32).and(timestamp.ge(from as i32)))
        .filter(status.eq_any(&BLOCKED_STATUSES))
        .count()
        .first::<i64>(db)
        .context(ErrorKind::FtlDatabase)?;

    Ok(count as usize)
}

/// Get the number of unique domains in the specified time range
fn get_unique_domain_count(db: &SqliteConnection, from: u64, until: u64) -> Result<usize, Error> {
    use crate::databases::ftl::queries::dsl::*;
    use diesel::{dsl::sql, sql_types::BigInt};

    let count = queries
        // Count the number of distinct (unique) domains. Diesel does not seem
        // to support this kind of COUNT expression, so raw SQL must be used.
        .select(sql::<BigInt>("COUNT(DISTINCT domain)"))
        .filter(timestamp.le(until as i32).and(timestamp.ge(from as i32)))
        .first::<i64>(db)
        .context(ErrorKind::FtlDatabase)?;

    Ok(count as usize)
}

/// Get the number of queries with the specified query status in the specified
/// time range
pub fn get_query_status_count(
    db: &SqliteConnection,
    from: u64,
    until: u64,
    status_type: FtlQueryStatus
) -> Result<usize, Error> {
    use crate::databases::ftl::queries::dsl::*;

    let count = queries
        .filter(timestamp.le(until as i32).and(timestamp.ge(from as i32)))
        .filter(status.eq(status_type as i32))
        .count()
        .first::<i64>(db)
        .context(ErrorKind::FtlDatabase)?;

    Ok(count as usize)
}

#[cfg(test)]
mod test {
    use super::{
        get_blocked_query_count, get_query_status_count, get_summary_impl, get_unique_domain_count
    };
    use crate::{
        databases::ftl::connect_to_test_db,
        env::{Config, Env},
        ftl::FtlQueryStatus,
        routes::stats::summary::{ReplyTypes, Summary, TotalQueries}
    };
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 0;
    const UNTIL_TIMESTAMP: u64 = 177_180;

    /// Verify that the summary returned using the database is accurate
    #[test]
    fn summary_impl() {
        let expected_summary = Summary {
            gravity_size: 0,
            total_queries: TotalQueries {
                A: 36,
                AAAA: 35,
                ANY: 0,
                SRV: 0,
                SOA: 0,
                PTR: 23,
                TXT: 0
            },
            blocked_queries: 0,
            percent_blocked: 0f64,
            unique_domains: 11,
            forwarded_queries: 26,
            cached_queries: 28,
            reply_types: ReplyTypes {
                IP: 0,
                CNAME: 0,
                DOMAIN: 0,
                NODATA: 0,
                NXDOMAIN: 0
            },
            total_clients: 0,
            active_clients: 0,
            status: "enabled"
        };

        let db = connect_to_test_db();
        let env = Env::Test(Config::default(), HashMap::new());
        let actual_summary = get_summary_impl(FROM_TIMESTAMP, UNTIL_TIMESTAMP, &db, &env).unwrap();

        assert_eq!(actual_summary, expected_summary);
    }

    /// Verify the blocked query count is accurate
    #[test]
    fn blocked_query_count() {
        let expected = 0;

        let db = connect_to_test_db();
        let actual = get_blocked_query_count(&db, FROM_TIMESTAMP, UNTIL_TIMESTAMP).unwrap();

        assert_eq!(actual, expected);
    }

    /// Verify the unique domain count is accurate
    #[test]
    fn unique_domain_count() {
        let expected = 11;

        let db = connect_to_test_db();
        let actual = get_unique_domain_count(&db, FROM_TIMESTAMP, UNTIL_TIMESTAMP).unwrap();

        assert_eq!(actual, expected);
    }

    /// Verify the query status count is accurate
    #[test]
    fn query_status_count() {
        let expected = 26;

        let db = connect_to_test_db();
        let actual = get_query_status_count(
            &db,
            FROM_TIMESTAMP,
            UNTIL_TIMESTAMP,
            FtlQueryStatus::Forward
        )
        .unwrap();

        assert_eq!(actual, expected);
    }
}
