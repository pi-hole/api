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
use util::{reply_data, Reply};

/// Get the query types
#[get("/stats/query_types")]
pub fn query_types(_auth: User, ftl_memory: State<FtlMemory>) -> Reply {
    let counters = ftl_memory.counters()?;

    reply_data(json!([
        {
            "name": "A",
            "count": counters.query_type(FtlQueryType::A)
        },
        {
            "name": "AAAA",
            "count": counters.query_type(FtlQueryType::AAAA)
        },
        {
            "name": "ANY",
            "count": counters.query_type(FtlQueryType::ANY)
        },
        {
            "name": "SRV",
            "count": counters.query_type(FtlQueryType::SRV)
        },
        {
            "name": "SOA",
            "count": counters.query_type(FtlQueryType::SOA)
        },
        {
            "name": "PTR",
            "count": counters.query_type(FtlQueryType::PTR)
        },
        {
            "name": "TXT",
            "count": counters.query_type(FtlQueryType::TXT)
        }
    ]))
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
            strings: HashMap::new(),
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
