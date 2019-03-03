// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Upstream Servers Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlUpstream},
    routes::auth::User,
    util::{reply_data, Reply}
};
use rocket::State;

/// Get the upstreams
#[get("/stats/upstreams")]
pub fn upstreams(_auth: User, ftl_memory: State<FtlMemory>) -> Reply {
    let lock = ftl_memory.lock()?;
    let ftl_upstreams = ftl_memory.upstreams(&lock)?;
    let strings = ftl_memory.strings(&lock)?;
    let counters = ftl_memory.counters(&lock)?;

    // Get an array of valid upstream references (FTL allocates more than it uses)
    let mut ftl_upstreams: Vec<&FtlUpstream> = ftl_upstreams
        .iter()
        .take(counters.total_upstreams as usize)
        // Remove upstreams with a zero count
        .filter(|upstream| upstream.query_count > 0)
        .collect();

    // Sort the upstreams
    ftl_upstreams.sort_by(|a, b| b.query_count.cmp(&a.query_count));

    let mut upstreams: Vec<UpstreamItemReply> = Vec::with_capacity(ftl_upstreams.len() + 2);

    // Add blocklist and cache upstreams
    upstreams.push(UpstreamItemReply {
        name: "blocklist".to_owned(),
        ip: "blocklist".to_owned(),
        count: counters.blocked_queries as usize
    });
    upstreams.push(UpstreamItemReply {
        name: "cache".to_owned(),
        ip: "cache".to_owned(),
        count: counters.cached_queries as usize
    });

    // Map the upstreams into the output format
    upstreams.extend(ftl_upstreams.into_iter().map(|upstream| {
        let ip = upstream.get_ip(&strings).to_owned();
        let name = upstream.get_name(&strings).unwrap_or_default().to_owned();

        UpstreamItemReply {
            name,
            ip,
            count: upstream.query_count as usize
        }
    }));

    reply_data(UpstreamsReply {
        upstreams,
        forwarded_queries: counters.forwarded_queries as usize,
        total_queries: counters.total_queries as usize
    })
}

/// Represents the reply structure for returning upstream item data
#[derive(Serialize)]
pub struct UpstreamItemReply {
    pub name: String,
    pub ip: String,
    pub count: usize
}

/// Represents the reply structure for upstreams endpoints
#[derive(Serialize)]
pub struct UpstreamsReply {
    pub upstreams: Vec<UpstreamItemReply>,
    pub forwarded_queries: usize,
    pub total_queries: usize
}

#[cfg(test)]
mod test {
    use crate::{
        ftl::{FtlCounters, FtlMemory, FtlSettings, FtlUpstream},
        testing::TestBuilder
    };
    use std::collections::HashMap;

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
    /// (they are still shown though)
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
                domains: Vec::new(),
                over_time: Vec::new(),
                queries: Vec::new(),
                settings: FtlSettings::default()
            })
            .expect_json(json!({
                "upstreams": [
                    { "name": "blocklist", "ip": "blocklist", "count": 0 },
                    { "name": "cache", "ip": "cache", "count": 0 },
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
                domains: Vec::new(),
                over_time: Vec::new(),
                queries: Vec::new(),
                settings: FtlSettings::default()
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
