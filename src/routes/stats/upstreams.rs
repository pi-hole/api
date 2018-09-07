// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Forward Destinations Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use ftl::{FtlMemory, FtlUpstream};
use rocket::State;
use rocket_contrib::Value;
use util::{reply_data, Reply};

/// Get the upstreams
#[get("/stats/upstreams")]
pub fn upstreams(_auth: User, ftl_memory: State<FtlMemory>) -> Reply {
    let upstreams = ftl_memory.upstreams()?;
    let strings = ftl_memory.strings()?;
    let counters = ftl_memory.counters()?;

    // Get an array of valid upstream references (FTL allocates more than it uses)
    let mut upstreams: Vec<&FtlUpstream> = upstreams
        .iter()
        .take(counters.total_upstreams as usize)
        .collect();

    // Remove upstreams with a zero count
    upstreams.retain(|upstream| upstream.query_count > 0);

    // Sort the upstreams
    upstreams.sort_by(|a, b| b.query_count.cmp(&a.query_count));

    // Map the upstreams into the output format
    let mut upstreams: Vec<Value> = upstreams
        .into_iter()
        .map(|upstream| {
            let ip = upstream.get_ip(&strings);
            let name = upstream.get_name(&strings).unwrap_or_default();

            json!({
                "name": name,
                "ip": ip,
                "count": upstream.query_count
            })
        })
        .collect();

    // Add cache and blocklist upstreams
    if counters.cached_queries > 0 {
        upstreams.insert(
            0,
            json!({
                "name": "cache",
                "ip": "cache",
                "count": counters.cached_queries
            })
        );
    }

    if counters.blocked_queries > 0 {
        upstreams.insert(
            0,
            json!({
                "name": "blocklist",
                "ip": "blocklist",
                "count": counters.blocked_queries
            })
        );
    }

    reply_data(json!({
        "upstreams": upstreams,
        "forwarded_queries": counters.forwarded_queries,
        "total_queries": counters.total_queries
    }))
}

#[cfg(test)]
mod test {
    use ftl::{FtlCounters, FtlMemory, FtlUpstream};
    use std::collections::HashMap;
    use testing::TestBuilder;

    fn test_upstream_data() -> (Vec<FtlUpstream>, HashMap<usize, String>) {
        let mut strings = HashMap::new();
        strings.insert(1, "8.8.8.8".to_owned());
        strings.insert(2, "google-public-dns-a.google.com".to_owned());
        strings.insert(3, "8.8.4.4".to_owned());
        strings.insert(4, "google-public-dns-b.google.com".to_owned());
        strings.insert(5, "1.1.1.1".to_owned());

        let upstreams = vec![
            FtlUpstream::new(10, 0, 1, Some(2)),
            FtlUpstream::new(4, 0, 3, Some(4)),
            FtlUpstream::new(3, 0, 5, None),
        ];

        (upstreams, strings)
    }

    /// Get the upstreams when there have been no blocked or cached queries
    /// (just the real upstreams)
    #[test]
    fn no_blocked_or_cached() {
        let (upstreams, strings) = test_upstream_data();

        TestBuilder::new()
            .endpoint("/admin/api/stats/upstreams")
            .ftl_memory(FtlMemory::Test {
                upstreams,
                strings,
                counters: FtlCounters {
                    total_upstreams: 3,
                    total_queries: 17,
                    forwarded_queries: 17,
                    ..FtlCounters::default()
                },
                clients: Vec::new(),
                domains: Vec::new()
            })
            .expect_json(json!({
                "upstreams": [
                    { "name": "google-public-dns-a.google.com", "ip": "8.8.8.8", "count": 10 },
                    { "name": "google-public-dns-b.google.com", "ip": "8.8.4.4", "count": 4 },
                    { "name": "", "ip": "1.1.1.1", "count": 3 }
                ],
                "total_queries": 17,
                "forwarded_queries": 17
            }))
            .test();
    }

    /// Get the upstreams when there have been blocked and cached queries
    /// (including the pseudo-upstreams)
    #[test]
    fn with_blocked_and_cached() {
        let (upstreams, strings) = test_upstream_data();

        TestBuilder::new()
            .endpoint("/admin/api/stats/upstreams")
            .ftl_memory(FtlMemory::Test {
                upstreams,
                strings,
                counters: FtlCounters {
                    total_upstreams: 3,
                    total_queries: 19,
                    forwarded_queries: 17,
                    blocked_queries: 1,
                    cached_queries: 1,
                    ..FtlCounters::default()
                },
                clients: Vec::new(),
                domains: Vec::new()
            })
            .expect_json(json!({
                "upstreams": [
                    { "name": "blocklist", "ip": "blocklist", "count": 1 },
                    { "name": "cache", "ip": "cache", "count": 1 },
                    { "name": "google-public-dns-a.google.com", "ip": "8.8.8.8", "count": 10 },
                    { "name": "google-public-dns-b.google.com", "ip": "8.8.4.4", "count": 4 },
                    { "name": "", "ip": "1.1.1.1", "count": 3 }
                ],
                "total_queries": 19,
                "forwarded_queries": 17
            }))
            .test();
    }
}
