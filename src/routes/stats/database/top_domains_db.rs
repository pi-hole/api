// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Domains/Blocked Endpoints - DB Version
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    env::{Env, PiholeFile},
    ftl::BLOCKED_STATUSES,
    routes::{
        auth::User,
        stats::{
            check_privacy_level_top_domains, check_query_log_show_top_domains,
            common::{get_excluded_domains, get_hidden_domain},
            database::{
                query_types_db::get_query_type_counts, summary_db::get_blocked_query_count
            },
            top_domains::{TopDomainItemReply, TopDomainParams, TopDomainsReply}
        }
    },
    util::{reply_data, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, sqlite::SqliteConnection};
use failure::ResultExt;
use rocket::{request::Form, State};

/// Return the top domains
#[get("/stats/database/top_domains?<from>&<until>&<params..>")]
pub fn top_domains_db(
    _auth: User,
    env: State<Env>,
    db: FtlDatabase,
    from: u64,
    until: u64,
    params: Form<TopDomainParams>
) -> Reply {
    reply_data(top_domains_db_impl(
        &env,
        &db as &SqliteConnection,
        from,
        until,
        params.into_inner()
    )?)
}

/// Return the top domains
fn top_domains_db_impl(
    env: &Env,
    db: &SqliteConnection,
    from: u64,
    until: u64,
    params: TopDomainParams
) -> Result<TopDomainsReply, Error> {
    // Resolve the parameters
    let limit = params.limit.unwrap_or(10);
    let audit = params.audit.unwrap_or(false);
    let ascending = params.ascending.unwrap_or(false);
    let blocked = params.blocked.unwrap_or(false);

    // Check if we are allowed to share the top domains
    if let Some(reply) = check_query_log_show_top_domains(env, blocked)? {
        // We can not share any of the domains, so use the reply returned by the
        // function
        return Ok(reply);
    }

    let total_count = if blocked {
        get_blocked_query_count(db, from, until)?
    } else {
        // Total query count is the sum of all query type counts
        get_query_type_counts(db, from, until)?.values().sum()
    } as usize;

    // Check if the domain details are private
    if let Some(reply) = check_privacy_level_top_domains(env, blocked, total_count)? {
        // We can not share any of the domains, so use the reply returned by the
        // function
        return Ok(reply);
    }

    // Find domains which should not be considered
    let ignored_domains = get_ignored_domains(env, audit)?;

    // Fetch the top domains and map into the reply structure
    let top_domains: Vec<TopDomainItemReply> =
        execute_top_domains_query(db, from, until, ignored_domains, blocked, ascending, limit)?
            .into_iter()
            .map(|(domain, count)| TopDomainItemReply {
                domain,
                count: count as usize
            })
            .collect();

    // Output format changes when getting top blocked domains
    if blocked {
        Ok(TopDomainsReply {
            top_domains,
            total_queries: None,
            blocked_queries: Some(total_count)
        })
    } else {
        Ok(TopDomainsReply {
            top_domains,
            total_queries: Some(total_count),
            blocked_queries: None
        })
    }
}

/// Get the list of domains to ignore. If the audit flag is true, audited
/// domains are ignored (only show unaudited domains).
fn get_ignored_domains(env: &Env, audit: bool) -> Result<Vec<String>, Error> {
    // Ignore domains excluded via SetupVars
    let mut ignored_domains = get_excluded_domains(env)?;

    // Ignore the hidden domain (due to privacy level)
    ignored_domains.push(get_hidden_domain().to_owned());

    // If audit flag is true, only include unaudited domains
    if audit {
        ignored_domains.extend(env.read_file_lines(PiholeFile::AuditLog)?);
    }

    Ok(ignored_domains)
}

/// Create and execute the database query to retrieve the top domain details.
/// The returned Vec contains each domain and its count, sorted and ordered
/// according to the parameters.
fn execute_top_domains_query(
    db: &SqliteConnection,
    from: u64,
    until: u64,
    ignored_domains: Vec<String>,
    blocked: bool,
    ascending: bool,
    limit: usize
) -> Result<Vec<(String, i64)>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // Create query
    let db_query = queries
        .select((domain, sql::<BigInt>("COUNT(*)")))
        // Only consider queries in the time interval
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.le(until as i32))
        // Filter out ignored domains
        .filter(domain.ne_all(ignored_domains))
        // Group queries by domain
        .group_by(domain)
        // Take into account limit
        .limit(limit as i64)
        // Box the query so we can conditionally modify it
        .into_boxed();

    // Set the sort order
    let db_query = if ascending {
        db_query.order((sql::<BigInt>("COUNT(*)").asc(), domain))
    } else {
        db_query.order((sql::<BigInt>("COUNT(*)").desc(), domain))
    };

    // Filter by status
    let db_query = if blocked {
        db_query.filter(status.eq_any(&BLOCKED_STATUSES))
    } else {
        db_query.filter(status.ne_all(&BLOCKED_STATUSES))
    };

    // Execute query
    Ok(db_query
        .load::<(String, i64)>(db)
        .context(ErrorKind::FtlDatabase)?)
}
