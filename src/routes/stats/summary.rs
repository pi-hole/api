// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Summary Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::Env;
use ftl::{FtlMemory, FtlPrivacyLevel, FtlQueryType};
use rocket::State;
use settings::{ConfigEntry, FtlConfEntry, SetupVarsEntry};
use util::{reply_data, Reply};

/// Get the summary data
#[get("/stats/summary")]
pub fn get_summary(ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    let counters = ftl_memory.counters()?;

    let percent_blocked = if counters.total_queries == 0 {
        0.0
    } else {
        (counters.blocked_queries * 100) as f64 / counters.total_queries as f64
    };

    let (total_clients, active_clients) = {
        if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)?
            >= FtlPrivacyLevel::HideDomainsAndClients
        {
            // If clients are supposed to be hidden, pretend there are no clients
            (0, 0)
        } else {
            // Only show active clients, and ignore hidden clients
            let clients = ftl_memory.clients()?;
            let strings = ftl_memory.strings()?;

            let hidden_client_count = clients
                .iter()
                .filter(|client| client.get_ip(&strings) == "0.0.0.0")
                .count();

            let active_client_count = clients
                .iter()
                .filter(|client| client.query_count > 0 && client.get_ip(&strings) != "0.0.0.0")
                .count();

            (
                counters.total_clients as usize - hidden_client_count,
                active_client_count
            )
        }
    };

    let status = if SetupVarsEntry::BlockingEnabled.read_as(&env)? {
        "enabled"
    } else {
        "disabled"
    };

    reply_data(json!({
        "gravity_size": counters.gravity_size,
        "total_queries": {
            "A": counters.query_type(FtlQueryType::A),
            "AAAA": counters.query_type(FtlQueryType::AAAA),
            "ANY": counters.query_type(FtlQueryType::ANY),
            "SRV": counters.query_type(FtlQueryType::SRV),
            "SOA": counters.query_type(FtlQueryType::SOA),
            "PTR": counters.query_type(FtlQueryType::PTR),
            "TXT": counters.query_type(FtlQueryType::TXT)
        },
        "blocked_queries": counters.blocked_queries,
        "percent_blocked": percent_blocked,
        "unique_domains": counters.total_domains,
        "forwarded_queries": counters.forwarded_queries,
        "cached_queries": counters.cached_queries,
        "reply_types": {
            "IP": counters.reply_count_ip,
            "CNAME": counters.reply_count_cname,
            "DOMAIN": counters.reply_count_domain,
            "NODATA": counters.reply_count_nodata,
            "NXDOMAIN": counters.reply_count_nxdomain
        },
        "total_clients": total_clients,
        "active_clients": active_clients,
        "status": status
    }))
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use ftl::{FtlClient, FtlCounters, FtlMemory};
    use std::collections::HashMap;
    use testing::TestBuilder;

    /// There are 6 clients, two inactive, one hidden, and two with names.
    fn test_data() -> FtlMemory {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "10.1.1.3".to_owned());
        strings.insert(5, "client3".to_owned());
        strings.insert(6, "10.1.1.4".to_owned());
        strings.insert(7, "10.1.1.5".to_owned());
        strings.insert(8, "0.0.0.0".to_owned());

        FtlMemory::Test {
            clients: vec![
                FtlClient::new(1, 0, 1, Some(2)),
                FtlClient::new(1, 0, 3, None),
                FtlClient::new(1, 0, 4, Some(5)),
                FtlClient::new(1, 0, 6, None),
                FtlClient::new(0, 0, 7, None),
                FtlClient::new(0, 0, 8, None),
            ],
            strings,
            counters: FtlCounters {
                gravity_size: 100_000,
                total_queries: 7,
                query_type_counters: [3, 4, 1, 0, 0, 3, 0],
                blocked_queries: 2,
                total_domains: 6,
                forwarded_queries: 3,
                cached_queries: 2,
                reply_count_ip: 3,
                reply_count_cname: 3,
                reply_count_domain: 1,
                reply_count_nodata: 1,
                reply_count_nxdomain: 2,
                total_clients: 6,
                ..FtlCounters::default()
            }
        }
    }

    #[test]
    fn enabled_and_no_privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/summary")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
            .expect_json(json!({
                "gravity_size": 100_000,
                "total_queries": {
                    "A": 3,
                    "AAAA": 4,
                    "ANY": 1,
                    "SRV": 0,
                    "SOA": 0,
                    "PTR": 3,
                    "TXT": 0
                },
                "blocked_queries": 2,
                "percent_blocked": 28.571428571428577,
                "unique_domains": 6,
                "forwarded_queries": 3,
                "cached_queries": 2,
                "reply_types": {
                    "IP": 3,
                    "CNAME": 3,
                    "DOMAIN": 1,
                    "NODATA": 1,
                    "NXDOMAIN": 2
                },
                "total_clients": 5,
                "active_clients": 4,
                "status": "enabled"
            }))
            .test();
    }

    #[test]
    fn disabled_and_privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/summary")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=false")
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!({
                "gravity_size": 100_000,
                "total_queries": {
                    "A": 3,
                    "AAAA": 4,
                    "ANY": 1,
                    "SRV": 0,
                    "SOA": 0,
                    "PTR": 3,
                    "TXT": 0
                },
                "blocked_queries": 2,
                "percent_blocked": 28.571428571428577,
                "unique_domains": 6,
                "forwarded_queries": 3,
                "cached_queries": 2,
                "reply_types": {
                    "IP": 3,
                    "CNAME": 3,
                    "DOMAIN": 1,
                    "NODATA": 1,
                    "NXDOMAIN": 2
                },
                "total_clients": 0,
                "active_clients": 0,
                "status": "disabled"
            }))
            .test();
    }
}
