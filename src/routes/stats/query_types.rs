// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query Types Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use ftl::{FtlMemory, FtlQueryType};
use rocket::State;
use rocket_contrib::Value;
use util::{reply_data, Reply};

/// Get the query types
#[get("/stats/query_types")]
pub fn query_types(_auth: User, ftl_memory: State<FtlMemory>) -> Reply {
    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

    reply_data(
        FtlQueryType::variants()
            .iter()
            .map(|&variant| {
                json!({
                    "name": format!("{:?}", variant),
                    "count": counters.query_type(variant)
                })
            })
            .collect::<Vec<Value>>()
    )
}

#[cfg(test)]
mod test {
    use ftl::{FtlCounters, FtlMemory};
    use std::collections::HashMap;
    use testing::TestBuilder;

    fn test_data() -> FtlMemory {
        FtlMemory::Test {
            counters: FtlCounters {
                query_type_counters: [2, 2, 1, 1, 1, 2, 1],
                total_queries: 10,
                ..FtlCounters::default()
            },
            domains: Vec::new(),
            over_time: Vec::new(),
            over_time_clients: Vec::new(),
            strings: HashMap::new(),
            upstreams: Vec::new(),
            queries: Vec::new(),
            clients: Vec::new()
        }
    }

    /// Simple test to validate output
    #[test]
    fn query_types() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/query_types")
            .ftl_memory(test_data())
            .expect_json(json!([
                { "name": "A", "count": 2 },
                { "name": "AAAA", "count": 2 },
                { "name": "ANY", "count": 1 },
                { "name": "SRV", "count": 1 },
                { "name": "SOA", "count": 1 },
                { "name": "PTR", "count": 2 },
                { "name": "TXT", "count": 1 }
            ]))
            .test();
    }
}
