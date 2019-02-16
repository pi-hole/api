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
        stats::summary::{ReplyTypes, Summary, TotalQueries}
    },
    settings::{ConfigEntry, SetupVarsEntry},
    util::{reply_data, Error, ErrorKind, Reply}
};
use diesel::prelude::*;
use failure::ResultExt;
use rocket::State;
use std::collections::HashMap;

/// Get summary data from database
#[get("/stats/database/summary?<from>&<until>")]
pub fn get_summary_db(
    from: u64,
    until: u64,
    _auth: User,
    ftl_database: FtlDatabase,
    env: State<Env>
) -> Reply {
    // Cast the database connection to &SqliteConnection to make using it easier
    // (only need to cast once, here)
    let db = &ftl_database as &SqliteConnection;

    reply_data(get_summary_impl(from, until, db, &env)?)
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

/// Get the number of queries with each query type in the specified time range
fn get_query_type_counts(
    db: &SqliteConnection,
    from: u64,
    until: u64
) -> Result<HashMap<FtlQueryType, usize>, Error> {
    use crate::databases::ftl::queries::dsl::*;
    use diesel::{dsl::sql, sql_types::BigInt};

    let counts = queries
        // Select the query types and their counts.
        // The raw SQL is used due to a limitation of Diesel, in that it doesn't
        // have full support for mixing aggregate and non-aggregate data when
        // using group_by. See https://github.com/diesel-rs/diesel/issues/1781
        .select((query_type, sql::<BigInt>("COUNT(*)")))
        // Search in the specified time interval
        .filter(timestamp.le(until as i32).and(timestamp.ge(from as i32)))
        // Group the results by query type
        .group_by(query_type)
        // Execute the query
        .get_results::<(i32, i64)>(db)
        // Add error context and check for errors
        .context(ErrorKind::FtlDatabase)?
        // Turn the resulting Vec into an iterator
        .into_iter()
        // Map the values into (FtlQueryType, usize)
        .map(|(q_type, count)| {
            (FtlQueryType::from_number(q_type as isize).unwrap(), count as usize)
        })
        // Turn the iterator into a HashMap
        .collect();

    Ok(counts)
}

/// Get the number of blocked queries in the specified time range
fn get_blocked_query_count(db: &SqliteConnection, from: u64, until: u64) -> Result<usize, Error> {
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
fn get_query_status_count(
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
