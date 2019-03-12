// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Upstream Servers Endpoint - DB Version
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    ftl::FtlQueryStatus,
    routes::{
        auth::User,
        stats::{
            database::{get_blocked_query_count, get_query_status_count},
            upstreams::{UpstreamItemReply, UpstreamsReply}
        }
    },
    util::{reply_result, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, sqlite::SqliteConnection};
use failure::ResultExt;
use std::collections::HashMap;

/// Get upstream data from the database
#[get("/stats/database/upstreams?<from>&<until>")]
pub fn upstreams_db(from: u64, until: u64, _auth: User, db: FtlDatabase) -> Reply {
    reply_result(upstreams_db_impl(from, until, &db as &SqliteConnection))
}

/// Get upstream data from the database
fn upstreams_db_impl(
    from: u64,
    until: u64,
    db: &SqliteConnection
) -> Result<UpstreamsReply, Error> {
    let upstream_counts = get_upstream_counts(from, until, db)?;
    let blocked_count = get_blocked_query_count(db, from, until)?;
    let cached_count = get_query_status_count(db, from, until, FtlQueryStatus::Cache)?;

    // Total queries is the sum of the upstream counts
    let total_queries = upstream_counts.values().sum::<i64>() as usize;
    // Forwarded queries are the sum of all upstream counts where the upstream is
    // not null
    let forwarded_queries = total_queries - upstream_counts[&None] as usize;

    // Capacity is the number of upstreams plus 1 for blocklists and 1 for
    // cache. upstream_counts.len() equals the number of upstreams plus 1
    // (blocklists and cache), so we just need to add one more slot.
    let mut upstreams = Vec::with_capacity(upstream_counts.len() + 1);

    // Add blocklist and cache upstreams
    upstreams.push(UpstreamItemReply {
        name: "blocklist".to_owned(),
        ip: "blocklist".to_owned(),
        count: blocked_count
    });
    upstreams.push(UpstreamItemReply {
        name: "cache".to_owned(),
        ip: "cache".to_owned(),
        count: cached_count
    });

    // Convert the upstreams into the reply structs
    let mut upstream_counts: Vec<UpstreamItemReply> = upstream_counts
        .into_iter()
        .filter_map(|(ip, count)| {
            if let Some(ip) = ip {
                Some(UpstreamItemReply {
                    name: "".to_owned(),
                    ip,
                    count: count as usize
                })
            } else {
                // Ignore the blocked and cached queries. These have already
                // been added above
                None
            }
        })
        .collect();

    // Sort the upstreams (descending by count)
    upstream_counts.sort_by(|a, b| b.count.cmp(&a.count));

    // Add the upstreams to the final list
    upstreams.extend(upstream_counts.into_iter());

    Ok(UpstreamsReply {
        upstreams,
        total_queries,
        forwarded_queries
    })
}

/// Get the number of queries for each upstream in the specified interval from
/// the database. Queries with no upstream (`None`) were either cached or
/// blocked.
fn get_upstream_counts(
    from: u64,
    until: u64,
    db: &SqliteConnection
) -> Result<HashMap<Option<String>, i64>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    Ok(queries
        .select((upstream, sql::<BigInt>("COUNT(*)")))
        // Search in the specified time interval
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.le(until as i32))
        // Group the results by upstream
        .group_by(upstream)
        // Execute the query
        .get_results::<(Option<String>, i64)>(db)
        // Add error context and check for errors
        .context(ErrorKind::FtlDatabase)?
        // Turn the resulting Vec into a HashMap
        .into_iter()
        .collect())
}

#[cfg(test)]
mod test {
    use super::{get_upstream_counts, upstreams_db_impl};
    use crate::{
        databases::ftl::connect_to_test_db,
        routes::stats::upstreams::{UpstreamItemReply, UpstreamsReply}
    };
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 0;
    const UNTIL_TIMESTAMP: u64 = 177_180;

    /// Verify that the upstream data returned using the database is accurate
    #[test]
    fn upstreams_impl() {
        let expected = UpstreamsReply {
            upstreams: vec![
                UpstreamItemReply {
                    name: "blocklist".to_owned(),
                    ip: "blocklist".to_owned(),
                    count: 0
                },
                UpstreamItemReply {
                    name: "cache".to_owned(),
                    ip: "cache".to_owned(),
                    count: 28
                },
                UpstreamItemReply {
                    name: "".to_owned(),
                    ip: "8.8.4.4".to_owned(),
                    count: 22
                },
                UpstreamItemReply {
                    name: "".to_owned(),
                    ip: "8.8.8.8".to_owned(),
                    count: 4
                },
            ],
            total_queries: 94,
            forwarded_queries: 26
        };

        let db = connect_to_test_db();
        let actual = upstreams_db_impl(FROM_TIMESTAMP, UNTIL_TIMESTAMP, &db).unwrap();

        assert_eq!(actual, expected);
    }

    /// Verify that the upstream count data is accurate
    #[test]
    fn upstream_counts() {
        let mut expected: HashMap<Option<String>, i64> = HashMap::new();
        expected.insert(None, 68);
        expected.insert(Some("8.8.4.4".to_owned()), 22);
        expected.insert(Some("8.8.8.8".to_owned()), 4);

        let db = connect_to_test_db();
        let actual = get_upstream_counts(FROM_TIMESTAMP, UNTIL_TIMESTAMP, &db).unwrap();

        assert_eq!(actual, expected);
    }
}
