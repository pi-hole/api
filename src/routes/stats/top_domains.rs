// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Domains/Blocked Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Env, PiholeFile},
    ftl::{FtlDomain, FtlMemory},
    routes::{
        auth::User,
        stats::common::{remove_excluded_domains, remove_hidden_domains}
    },
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel, SetupVarsEntry},
    util::{reply_data, Error, Reply}
};
use rocket::{request::Form, State};

/// Return the top domains
#[get("/stats/top_domains?<params..>")]
pub fn top_domains(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: Form<TopDomainParams>
) -> Reply {
    reply_data(get_top_domains(&ftl_memory, &env, params.into_inner())?)
}

/// Represents the possible GET parameters for top (blocked) domains requests
#[derive(FromForm, Default)]
pub struct TopDomainParams {
    pub limit: Option<usize>,
    pub audit: Option<bool>,
    pub ascending: Option<bool>,
    pub blocked: Option<bool>
}

/// Represents the reply structure for top (blocked) domains
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TopDomainsReply {
    pub top_domains: Vec<TopDomainItemReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_queries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_queries: Option<usize>
}

/// Represents the reply structure for a top (blocked) domain item
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TopDomainItemReply {
    pub domain: String,
    pub count: usize
}

/// Get the top domains (blocked or not)
fn get_top_domains(
    ftl_memory: &FtlMemory,
    env: &Env,
    params: TopDomainParams
) -> Result<TopDomainsReply, Error> {
    // Resolve the parameters
    let limit = params.limit.unwrap_or(10);
    let audit = params.audit.unwrap_or(false);
    let ascending = params.ascending.unwrap_or(false);
    let blocked = params.blocked.unwrap_or(false);

    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

    // Check if we are allowed to share the top domains
    if let Some(reply) = check_query_log_show_top_domains(env, blocked)? {
        // We can not share any of the domains, so use the reply returned by the
        // function
        return Ok(reply);
    }

    let total_count = if blocked {
        counters.blocked_queries
    } else {
        counters.total_queries
    } as usize;

    // Check if the domain details are private
    if let Some(reply) = check_privacy_level_top_domains(env, blocked, total_count)? {
        // We can not share any of the domains, so use the reply returned by the
        // function
        return Ok(reply);
    }

    let domains = ftl_memory.domains(&lock)?;
    let strings = ftl_memory.strings(&lock)?;

    // Get an array of valid domain references (FTL allocates more than it uses)
    let mut domains: Vec<&FtlDomain> = domains
        .iter()
        .take(counters.total_domains as usize)
        .collect();

    // Remove excluded and hidden domains
    remove_excluded_domains(&mut domains, env, &strings)?;
    remove_hidden_domains(&mut domains, &strings);

    // Remove domains with a count of 0
    if blocked {
        domains.retain(|domain| domain.blocked_count > 0);
    } else {
        domains.retain(|domain| (domain.query_count - domain.blocked_count) > 0);
    }

    // If audit flag is true, only include unaudited domains
    if audit {
        let audited_domains = env.read_file_lines(PiholeFile::AuditLog)?;

        // Get a vector of references to strings, to better compare with the domains
        let audited_domains: Vec<&str> = audited_domains.iter().map(String::as_str).collect();

        domains.retain(|domain| !audited_domains.contains(&domain.get_domain(&strings)));
    }

    // Sort the domains (descending by default)
    match (ascending, blocked) {
        (false, false) => domains.sort_by(|a, b| {
            (b.query_count - b.blocked_count).cmp(&(a.query_count - a.blocked_count))
        }),
        (true, false) => domains.sort_by(|a, b| {
            (a.query_count - a.blocked_count).cmp(&(b.query_count - b.blocked_count))
        }),
        (false, true) => domains.sort_by(|a, b| b.blocked_count.cmp(&a.blocked_count)),
        (true, true) => domains.sort_by(|a, b| a.blocked_count.cmp(&b.blocked_count))
    }

    // Take into account the limit
    if limit < domains.len() {
        domains.split_off(limit);
    }

    // Map the domains into the output format
    let top_domains: Vec<TopDomainItemReply> = domains
        .iter()
        .map(|domain| {
            let name = domain.get_domain(&strings).to_owned();
            let count = if blocked {
                domain.blocked_count
            } else {
                domain.query_count - domain.blocked_count
            } as usize;

            TopDomainItemReply {
                domain: name,
                count
            }
        })
        .collect();

    // Output format changes when getting top blocked domains
    if blocked {
        Ok(TopDomainsReply {
            top_domains,
            total_queries: None,
            blocked_queries: Some(counters.blocked_queries as usize)
        })
    } else {
        Ok(TopDomainsReply {
            top_domains,
            total_queries: Some(counters.total_queries as usize),
            blocked_queries: None
        })
    }
}

/// Check the `API_QUERY_LOG_SHOW` setting with the requested top domains type
/// to see if any data can be shown. If no data can be shown (ex. the setting
/// equals `permittedonly` but top blocked domains are requested) then a reply
/// is returned which should be used as the endpoint reply.
pub fn check_query_log_show_top_domains(
    env: &Env,
    blocked: bool
) -> Result<Option<TopDomainsReply>, Error> {
    let display_setting = SetupVarsEntry::ApiQueryLogShow.read(env)?;

    if display_setting == "nothing"
        || (display_setting == "permittedonly" && blocked)
        || (display_setting == "blockedonly" && !blocked)
    {
        if blocked {
            return Ok(Some(TopDomainsReply {
                top_domains: Vec::new(),
                total_queries: None,
                blocked_queries: Some(0)
            }));
        } else {
            return Ok(Some(TopDomainsReply {
                top_domains: Vec::new(),
                total_queries: Some(0),
                blocked_queries: None
            }));
        }
    }

    Ok(None)
}

/// Check the privacy level to see if domains are allowed to be shared. If not,
/// then only return the relevant count (total or blocked queries).
pub fn check_privacy_level_top_domains(
    env: &Env,
    blocked: bool,
    count: usize
) -> Result<Option<TopDomainsReply>, Error> {
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)? >= FtlPrivacyLevel::HideDomains
    {
        if blocked {
            return Ok(Some(TopDomainsReply {
                top_domains: Vec::new(),
                total_queries: None,
                blocked_queries: Some(count)
            }));
        } else {
            return Ok(Some(TopDomainsReply {
                top_domains: Vec::new(),
                total_queries: Some(count),
                blocked_queries: None
            }));
        }
    }

    Ok(None)
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{FtlCounters, FtlDomain, FtlMemory, FtlRegexMatch, FtlSettings},
        testing::TestBuilder
    };
    use std::collections::HashMap;

    /// Four clients, one hidden, one with no queries
    fn test_data() -> FtlMemory {
        let mut strings = HashMap::new();
        strings.insert(1, "example.com".to_owned());
        strings.insert(2, "hidden".to_owned());
        strings.insert(3, "github.com".to_owned());
        strings.insert(4, "localhost".to_owned());
        strings.insert(5, "example.net".to_owned());

        FtlMemory::Test {
            domains: vec![
                FtlDomain::new(10, 10, 1, FtlRegexMatch::Unknown),
                FtlDomain::new(4, 2, 2, FtlRegexMatch::Unknown),
                FtlDomain::new(20, 0, 3, FtlRegexMatch::Unknown),
                FtlDomain::new(0, 0, 4, FtlRegexMatch::Unknown),
                FtlDomain::new(10, 9, 5, FtlRegexMatch::Unknown),
            ],
            clients: Vec::new(),
            over_time: Vec::new(),
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters {
                total_queries: 39,
                blocked_queries: 21,
                total_domains: 5,
                ..FtlCounters::default()
            },
            settings: FtlSettings::default()
        }
    }

    /// Show permitted domains, but no hidden, inactive, or completely blocked
    /// domains
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_domains": [
                    { "domain": "github.com", "count": 20 },
                    { "domain": "example.net", "count": 1 }
                ],
                "total_queries": 39
            }))
            .test();
    }

    /// Don't show more domains than the limit
    #[test]
    fn limit() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains?limit=1")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_domains": [
                    { "domain": "github.com", "count": 20 }
                ],
                "total_queries": 39
            }))
            .test();
    }

    /// Show blocked domains, but no hidden, inactive, or completely unblocked
    /// domains
    #[test]
    fn blocked() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains?blocked=true")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_domains": [
                    { "domain": "example.com", "count": 10 },
                    { "domain": "example.net", "count": 9 }
                ],
                "blocked_queries": 21
            }))
            .test();
    }

    /// Show permitted domains in ascending order, but no hidden, inactive, or
    /// completely blocked domains
    #[test]
    fn ascending() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains?ascending=true")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_domains": [
                    { "domain": "example.net", "count": 1 },
                    { "domain": "github.com", "count": 20 }
                ],
                "total_queries": 39
            }))
            .test();
    }

    /// Show unaudited domains in ascending order, but no hidden, inactive, or
    /// audited domains
    #[test]
    fn audit() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains?audit=true")
            .ftl_memory(test_data())
            .file(PiholeFile::AuditLog, "example.net")
            .expect_json(json!({
                "top_domains": [
                    { "domain": "github.com", "count": 20 }
                ],
                "total_queries": 39
            }))
            .test();
    }

    /// Show permitted domains, but no hidden, inactive, or excluded domains
    #[test]
    fn excluded() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_domains")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "API_EXCLUDE_DOMAINS=example.net")
            .expect_json(json!({
                "top_domains": [
                    { "domain": "github.com", "count": 20 }
                ],
                "total_queries": 39
            }))
            .test();
    }
}
