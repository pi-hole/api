// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// History Test Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{
        FtlClient, FtlCounters, FtlDnssecType, FtlDomain, FtlMemory, FtlQuery, FtlQueryReplyType,
        FtlQueryStatus, FtlQueryType, FtlRegexMatch, FtlSettings, FtlUpstream, MAGIC_BYTE
    },
    settings::FtlPrivacyLevel
};
use std::collections::HashMap;

/// Shorthand for making `FtlQuery` structs
macro_rules! query {
    (
            $id:expr,
            $database:expr,
            $qtype:ident,
            $status:ident,
            $domain:expr,
            $client:expr,
            $upstream:expr,
            $timestamp:expr,
            $private:ident
        ) => {
        FtlQuery {
            magic: MAGIC_BYTE,
            id: $id,
            database_id: $database,
            timestamp: $timestamp,
            time_index: 1,
            response_time: 1,
            domain_id: $domain,
            client_id: $client,
            upstream_id: $upstream,
            query_type: FtlQueryType::$qtype,
            status: FtlQueryStatus::$status,
            reply_type: FtlQueryReplyType::IP,
            dnssec_type: FtlDnssecType::Unspecified,
            is_complete: true,
            privacy_level: FtlPrivacyLevel::$private
        }
    };
}

/// Creates an `FtlMemory` struct from the other test data functions
pub fn test_memory() -> FtlMemory {
    FtlMemory::Test {
        clients: test_clients(),
        counters: test_counters(),
        domains: test_domains(),
        over_time: Vec::new(),
        strings: test_strings(),
        queries: test_queries(),
        upstreams: test_upstreams(),
        settings: FtlSettings::default()
    }
}

/// 9 queries. Query 9 is private. Last two are not in the database. Query 1
/// has a DNSSEC type of Secure and a reply type of CNAME. The database
/// timestamps end at 177180, so the in memory queries start at 263581
/// (24 hours after). The database ids end at 94, so the in memory database IDs
/// start at 95.
///
/// | ID |  DB | Type |   Status   | Domain | Client | Upstream | Timestamp |
/// | -- | --- | ---- | ---------- | ------ | ------ | -------- | --------- |
/// | 1  |  95 | A    | Forward    | 0      | 0      | 0        | 263581    |
/// | 2  |  96 | AAAA | Forward    | 0      | 0      | 0        | 263582    |
/// | 3  |  97 | PTR  | Forward    | 0      | 0      | 0        | 263583    |
/// | 4  |  98 | A    | Gravity    | 1      | 1      | 0        | 263583    |
/// | 5  |  99 | AAAA | Cache      | 0      | 1      | 0        | 263584    |
/// | 6  | 100 | AAAA | Wildcard   | 2      | 1      | 0        | 263585    |
/// | 7  | 101 | A    | Blacklist  | 3      | 2      | 0        | 263585    |
/// | 8  |   0 | AAAA | ExternalB. | 4      | 2      | 1        | 263586    |
/// | 9  |   0 | A    | Forward    | 5      | 3      | 0        | 263587    |
pub fn test_queries() -> Vec<FtlQuery> {
    vec![
        FtlQuery {
            magic: MAGIC_BYTE,
            id: 1,
            database_id: 95,
            timestamp: 263_581,
            time_index: 1,
            response_time: 1,
            domain_id: 0,
            client_id: 0,
            upstream_id: 0,
            query_type: FtlQueryType::A,
            status: FtlQueryStatus::Forward,
            reply_type: FtlQueryReplyType::CNAME,
            dnssec_type: FtlDnssecType::Secure,
            is_complete: true,
            privacy_level: FtlPrivacyLevel::ShowAll
        },
        query!(2, 96, AAAA, Forward, 0, 0, 0, 263_582, ShowAll),
        query!(3, 97, PTR, Forward, 0, 0, 0, 263_583, ShowAll),
        query!(4, 98, A, Gravity, 1, 1, 0, 263_583, ShowAll),
        query!(5, 99, AAAA, Cache, 0, 1, 0, 263_584, ShowAll),
        query!(6, 100, AAAA, Wildcard, 2, 1, 0, 263_585, ShowAll),
        query!(7, 101, A, Blacklist, 3, 2, 0, 263_585, ShowAll),
        query!(8, 0, AAAA, ExternalBlockIp, 4, 2, 1, 263_586, ShowAll),
        query!(9, 0, A, Forward, 5, 3, 0, 263_587, Maximum),
    ]
}

/// The counters necessary for the history tests.
pub fn test_counters() -> FtlCounters {
    FtlCounters {
        total_queries: 9,
        total_upstreams: 2,
        total_domains: 6,
        total_clients: 4,
        ..FtlCounters::default()
    }
}

/// 6 domains. See `test_queries` for how they're used.
pub fn test_domains() -> Vec<FtlDomain> {
    vec![
        FtlDomain::new(4, 0, 1, FtlRegexMatch::NotBlocked),
        FtlDomain::new(1, 1, 2, FtlRegexMatch::NotBlocked),
        FtlDomain::new(1, 1, 3, FtlRegexMatch::Blocked),
        FtlDomain::new(1, 1, 4, FtlRegexMatch::NotBlocked),
        FtlDomain::new(1, 0, 5, FtlRegexMatch::NotBlocked),
        FtlDomain::new(1, 0, 13, FtlRegexMatch::NotBlocked),
    ]
}

/// 4 clients. See `test_queries` for how they're used.
pub fn test_clients() -> Vec<FtlClient> {
    vec![
        FtlClient::new(3, 0, 6, Some(7)),
        FtlClient::new(3, 2, 8, None),
        FtlClient::new(2, 2, 9, None),
        FtlClient::new(1, 0, 10, None),
    ]
}

/// 1 upstream. See `test_queries` for how it's used.
pub fn test_upstreams() -> Vec<FtlUpstream> {
    vec![
        FtlUpstream::new(3, 0, 11, Some(12)),
        FtlUpstream::new(1, 0, 14, Some(15)),
    ]
}

/// Strings used in the test data
pub fn test_strings() -> HashMap<usize, String> {
    let mut strings = HashMap::new();
    strings.insert(1, "domain1.com".to_owned());
    strings.insert(2, "domain2.com".to_owned());
    strings.insert(3, "domain3.com".to_owned());
    strings.insert(4, "domain4.com".to_owned());
    strings.insert(5, "domain5.com".to_owned());
    strings.insert(6, "192.168.1.10".to_owned());
    strings.insert(7, "client1".to_owned());
    strings.insert(8, "192.168.1.11".to_owned());
    strings.insert(9, "192.168.1.12".to_owned());
    strings.insert(10, "0.0.0.0".to_owned());
    strings.insert(11, "8.8.8.8".to_owned());
    strings.insert(12, "google-public-dns-a.google.com".to_owned());
    strings.insert(13, "hidden".to_owned());
    strings.insert(14, "8.8.4.4".to_owned());
    strings.insert(15, "google-public-dns-b.google.com".to_owned());

    strings
}
