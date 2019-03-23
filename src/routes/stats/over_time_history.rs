// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Over Time Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::FtlMemory,
    routes::stats::common::get_current_over_time_slot,
    util::{reply_data, Reply}
};
use rocket::State;

/// Get the query history over time (separated into blocked and not blocked)
#[get("/stats/overTime/history")]
pub fn over_time_history(ftl_memory: State<FtlMemory>) -> Reply {
    let lock = ftl_memory.lock()?;
    let over_time = ftl_memory.over_time(&lock)?;

    let over_time_data: Vec<OverTimeItem> = over_time.iter()
        // Take all of the slots including the current slot
        .take(get_current_over_time_slot(&over_time) + 1)
        // Skip the overTime slots without any data
        .skip_while(|time| {
            (time.total_queries <= 0 && time.blocked_queries <= 0)
        })
        .map(|time| {
            OverTimeItem {
                timestamp: time.timestamp as u64,
                total_queries: time.total_queries as usize,
                blocked_queries: time.blocked_queries as usize
            }
        })
        .collect();

    reply_data(over_time_data)
}

#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct OverTimeItem {
    pub timestamp: u64,
    pub total_queries: usize,
    pub blocked_queries: usize
}

#[cfg(test)]
mod test {
    use crate::{
        ftl::{FtlCounters, FtlMemory, FtlOverTime, FtlSettings},
        testing::TestBuilder
    };
    use std::collections::HashMap;

    /// Data for testing over_time_history
    fn test_data() -> FtlMemory {
        FtlMemory::Test {
            over_time: vec![
                FtlOverTime::new(1, 1, 0, 0, 1, [0; 7]),
                FtlOverTime::new(2, 1, 1, 1, 0, [0; 7]),
                FtlOverTime::new(3, 0, 1, 0, 0, [0; 7]),
            ],
            counters: FtlCounters {
                ..FtlCounters::default()
            },
            clients: Vec::new(),
            upstreams: Vec::new(),
            strings: HashMap::new(),
            domains: Vec::new(),
            queries: Vec::new(),
            settings: FtlSettings::default()
        }
    }

    /// Default params will skip overTime slots until it finds the first slot
    /// with queries.
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/history")
            .ftl_memory(test_data())
            .expect_json(json!([
                { "timestamp": 1, "total_queries": 1, "blocked_queries": 0 },
                { "timestamp": 2, "total_queries": 1, "blocked_queries": 1 },
                { "timestamp": 3, "total_queries": 0, "blocked_queries": 1 }
            ]))
            .test();
    }
}
