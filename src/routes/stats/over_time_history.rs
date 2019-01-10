// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Over Time Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlMemory,
    settings::{ConfigEntry, FtlConfEntry},
    util::{reply_data, Reply}
};
use rocket::State;
use rocket_contrib::json::JsonValue;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get the query history over time (separated into blocked and not blocked)
#[get("/stats/overTime/history")]
pub fn over_time_history(ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;
    let over_time = ftl_memory.over_time(&lock)?;

    // Get the current timestamp, to be used when getting overTime data
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time web backwards")
        .as_secs() as f64;

    // Get the max log age FTL setting, to be used when getting overTime data
    let max_log_age = FtlConfEntry::MaxLogAge.read_as::<f64>(&env).unwrap_or(24.0) * 3600.0;

    let over_time_data: Vec<JsonValue> = over_time.iter()
       .take(counters.over_time_size as usize)
        // Skip the overTime slots without any data, and any slots which are
        // before the max-log-age time.
        .skip_while(|time| {
            (time.total_queries <= 0 && time.blocked_queries <= 0)
                || ((time.timestamp as f64) < timestamp - max_log_age)
        })
        .map(|time| {
            json!({
                "timestamp": time.timestamp,
                "total_queries": time.total_queries,
                "blocked_queries": time.blocked_queries
            })
        })
        .collect();

    reply_data(over_time_data)
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{FtlCounters, FtlMemory, FtlOverTime},
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
                over_time_size: 3,
                ..FtlCounters::default()
            },
            clients: Vec::new(),
            over_time_clients: Vec::new(),
            upstreams: Vec::new(),
            strings: HashMap::new(),
            domains: Vec::new(),
            queries: Vec::new()
        }
    }

    /// Default params will show overTime data from within the MAXLOGAGE
    /// timeframe, and will skip overTime slots until it finds the first slot
    /// with queries.
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/history")
            .ftl_memory(test_data())
            // Abuse From<&str> for f64 and use all overTime data
            .file(PiholeFile::FtlConfig, "MAXLOGAGE=inf")
            .expect_json(json!([
                { "timestamp": 1, "total_queries": 1, "blocked_queries": 0 },
                { "timestamp": 2, "total_queries": 1, "blocked_queries": 1 },
                { "timestamp": 3, "total_queries": 0, "blocked_queries": 1 }
            ]))
            .test();
    }

    /// Only overTime slots within the MAXLOGAGE value are considered
    #[test]
    fn max_log_age() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/history")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "MAXLOGAGE=0")
            .expect_json(json!([]))
            .test();
    }
}
