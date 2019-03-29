// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Summary Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::{FtlMemory, FtlQueryType},
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel, SetupVarsEntry},
    util::{reply_data, Reply}
};
use rocket::State;

/// Get the summary data
#[get("/stats/summary")]
pub fn get_summary(ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

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
            let clients = ftl_memory.clients(&lock)?;
            let strings = ftl_memory.strings(&lock)?;

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

    let status = if SetupVarsEntry::BlockingEnabled.is_true(&env)? {
        "enabled"
    } else {
        "disabled"
    };

    reply_data(Summary {
        gravity_size: counters.gravity_size as usize,
        total_queries: TotalQueries {
            A: counters.query_type(FtlQueryType::A),
            AAAA: counters.query_type(FtlQueryType::AAAA),
            ANY: counters.query_type(FtlQueryType::ANY),
            SRV: counters.query_type(FtlQueryType::SRV),
            SOA: counters.query_type(FtlQueryType::SOA),
            PTR: counters.query_type(FtlQueryType::PTR),
            TXT: counters.query_type(FtlQueryType::TXT)
        },
        blocked_queries: counters.blocked_queries as usize,
        percent_blocked,
        unique_domains: counters.total_domains as usize,
        forwarded_queries: counters.forwarded_queries as usize,
        cached_queries: counters.cached_queries as usize,
        reply_types: ReplyTypes {
            IP: counters.reply_count_ip as usize,
            CNAME: counters.reply_count_cname as usize,
            DOMAIN: counters.reply_count_domain as usize,
            NODATA: counters.reply_count_nodata as usize,
            NXDOMAIN: counters.reply_count_nxdomain as usize
        },
        total_clients,
        active_clients,
        status
    })
}

/// Represents the response of summary endpoints
#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct Summary {
    pub gravity_size: usize,
    pub total_queries: TotalQueries,
    pub blocked_queries: usize,
    pub percent_blocked: f64,
    pub unique_domains: usize,
    pub forwarded_queries: usize,
    pub cached_queries: usize,
    pub reply_types: ReplyTypes,
    pub total_clients: usize,
    pub active_clients: usize,
    pub status: &'static str
}

/// Part of the summary response
#[allow(non_snake_case)]
#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct TotalQueries {
    pub A: usize,
    pub AAAA: usize,
    pub ANY: usize,
    pub SRV: usize,
    pub SOA: usize,
    pub PTR: usize,
    pub TXT: usize
}

/// Part of the summary response
#[allow(non_snake_case)]
#[derive(Serialize)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ReplyTypes {
    pub IP: usize,
    pub CNAME: usize,
    pub DOMAIN: usize,
    pub NODATA: usize,
    pub NXDOMAIN: usize
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{FtlClient, FtlCounters, FtlMemory, FtlSettings},
        testing::TestBuilder
    };
    use std::collections::HashMap;

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
            domains: Vec::new(),
            over_time: Vec::new(),
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
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
            },
            settings: FtlSettings::default()
        }
    }

    #[test]
    fn enabled_and_no_privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/summary")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "BLOCKING_ENABLED=true")
            .file(PiholeFile::FtlConfig, "")
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
                "percent_blocked": 28.571_428_571_428_573,
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
                "percent_blocked": 28.571_428_571_428_573,
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
