// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Recent Blocked Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlMemory,
    routes::auth::User,
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Reply}
};
use rocket::{request::Form, State};

/// Get the `num` most recently blocked domains
#[get("/stats/recent_blocked?<params..>")]
pub fn recent_blocked(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: Form<RecentBlockedParams>
) -> Reply {
    get_recent_blocked(&ftl_memory, &env, params.num.unwrap_or(1))
}

/// Represents the possible GET parameters on `/stats/recent_blocked`
#[derive(FromForm)]
pub struct RecentBlockedParams {
    num: Option<usize>
}

/// Get `num`-many most recently blocked domains
pub fn get_recent_blocked(ftl_memory: &FtlMemory, env: &Env, num: usize) -> Reply {
    // Check if client details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)? >= FtlPrivacyLevel::HideDomains
    {
        return reply_data([0; 0]);
    }

    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;
    let queries = ftl_memory.queries(&lock)?;
    let strings = ftl_memory.strings(&lock)?;
    let domains = ftl_memory.domains(&lock)?;

    let recent_blocked: Vec<&str> = queries
        .iter()
        // Get the most recent queries first
        .rev()
        // Skip the uninitialized queries
        .skip(queries.len() - counters.total_queries as usize)
        // Only get blocked queries
        .filter(|query| query.is_blocked())
        // Get up to num queries
        .take(num)
        // Only return the domain
        .map(|query| domains[query.domain_id as usize].get_domain(&strings))
        .collect();

    reply_data(recent_blocked)
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{
            FtlCounters, FtlDnssecType, FtlDomain, FtlMemory, FtlQuery, FtlQueryReplyType,
            FtlQueryStatus, FtlQueryType, FtlRegexMatch, FtlSettings, MAGIC_BYTE
        },
        testing::TestBuilder
    };
    use std::collections::HashMap;

    /// Shorthand for making `FtlQuery` structs
    macro_rules! query {
        ($id:expr, $status:ident, $domain:expr) => {
            FtlQuery {
                magic: MAGIC_BYTE,
                id: $id,
                database_id: 0,
                timestamp: 1,
                time_index: 1,
                response_time: 1,
                domain_id: $domain,
                client_id: 0,
                upstream_id: 0,
                query_type: FtlQueryType::A,
                status: FtlQueryStatus::$status,
                reply_type: FtlQueryReplyType::IP,
                dnssec_type: FtlDnssecType::Unspecified,
                is_complete: true,
                is_private: false,
                ad_bit: false
            }
        };
    }

    /// 6 queries, 4 blocked
    fn test_queries() -> Vec<FtlQuery> {
        vec![
            query!(1, Forward, 0),
            query!(2, Gravity, 1),
            query!(3, Blacklist, 2),
            query!(4, Wildcard, 3),
            query!(5, ExternalBlock, 4),
            query!(6, Cache, 0),
        ]
    }

    /// 5 domain, 4 blocked, 1 regex blocked, 1 not blocked
    fn test_domains() -> Vec<FtlDomain> {
        vec![
            FtlDomain::new(2, 0, 1, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 1, 2, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 1, 3, FtlRegexMatch::NotBlocked),
            FtlDomain::new(1, 1, 4, FtlRegexMatch::Blocked),
            FtlDomain::new(1, 1, 5, FtlRegexMatch::NotBlocked),
        ]
    }

    /// Strings used in the test data
    fn test_strings() -> HashMap<usize, String> {
        let mut strings = HashMap::new();
        strings.insert(1, "domain1.com".to_owned());
        strings.insert(2, "domain2.com".to_owned());
        strings.insert(3, "domain3.com".to_owned());
        strings.insert(4, "domain4.com".to_owned());
        strings.insert(5, "domain5.com".to_owned());

        strings
    }

    /// Creates an `FtlMemory` struct from the other test data functions
    fn test_memory() -> FtlMemory {
        FtlMemory::Test {
            queries: test_queries(),
            domains: test_domains(),
            over_time: Vec::new(),
            strings: test_strings(),
            clients: Vec::new(),
            upstreams: Vec::new(),
            counters: FtlCounters {
                total_queries: 6,
                total_domains: 5,
                ..FtlCounters::default()
            },
            settings: FtlSettings::default()
        }
    }

    /// The default behavior shows one most recently blocked domain
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/recent_blocked")
            .ftl_memory(test_memory())
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!(["domain5.com"]))
            .test();
    }

    /// The `num` parameter returns that many most recently blocked domain
    #[test]
    fn multiple() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/recent_blocked?num=3")
            .ftl_memory(test_memory())
            .file(PiholeFile::FtlConfig, "")
            .expect_json(json!(["domain5.com", "domain4.com", "domain3.com"]))
            .test();
    }

    /// If there are less blocked domains than requested, return as many as we
    /// can find
    #[test]
    fn less_than_requested() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/recent_blocked?num=10")
            .file(PiholeFile::FtlConfig, "")
            .ftl_memory(test_memory())
            .expect_json(json!([
                "domain5.com",
                "domain4.com",
                "domain3.com",
                "domain2.com"
            ]))
            .test();
    }
}
