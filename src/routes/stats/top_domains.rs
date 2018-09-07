// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Domains/Blocked Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use env::PiholeFile;
use ftl::{FtlDomain, FtlMemory};
use rocket::State;
use rocket_contrib::Value;
use routes::stats::common::{remove_excluded_domains, remove_hidden_domains};
use settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel, SetupVarsEntry};
use std::io::{BufRead, BufReader};
use util::{reply_data, Reply};

/// Return the top domains with default parameters
#[get("/stats/top_domains")]
pub fn top_domains(_auth: User, ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    get_top_domains(&ftl_memory, &env, TopParams::default())
}

/// Return the top domains with specified parameters
#[get("/stats/top_domains?<params>")]
pub fn top_domains_params(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: TopParams
) -> Reply {
    get_top_domains(&ftl_memory, &env, params)
}

/// Represents the possible GET parameters on `/stats/top_domains` and
/// `/stats/top_blocked`
#[derive(FromForm)]
pub struct TopParams {
    limit: Option<usize>,
    audit: Option<bool>,
    ascending: Option<bool>,
    blocked: Option<bool>
}

impl Default for TopParams {
    /// The default parameters of top_domains and top_blocked requests
    fn default() -> Self {
        TopParams {
            limit: Some(10),
            audit: Some(false),
            ascending: Some(false),
            blocked: Some(false)
        }
    }
}

/// Get the top domains (blocked or not)
fn get_top_domains(ftl_memory: &FtlMemory, env: &Env, params: TopParams) -> Reply {
    let blocked = params.blocked.unwrap_or(false);
    let counters = ftl_memory.counters()?;
    let display_setting = SetupVarsEntry::ApiQueryLogShow.read(env)?;

    // Check if we are allowed to share this data (even the number of queries)
    if display_setting == "nothing"
        || (display_setting == "permittedonly" && blocked)
        || (display_setting == "blockedonly" && !blocked)
    {
        if blocked {
            return reply_data(json!({
                "top_domains": [],
                "blocked_queries": 0
            }));
        } else {
            return reply_data(json!({
                "top_domains": [],
                // If they requested permitted queries but they only want to
                // see blocked queries (and not nothing), then share the number
                // of blocked queries (total - permitted)
                "total_queries": if display_setting == "nothing" {
                    0
                } else {
                    counters.blocked_queries
                }
            }));
        }
    }

    // Check if the domain details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)? >= FtlPrivacyLevel::HideDomains
    {
        if blocked {
            return reply_data(json!({
                "top_domains": [],
                "blocked_queries": counters.blocked_queries
            }));
        } else {
            return reply_data(json!({
                "top_domains": [],
                "total_queries": counters.total_queries
            }));
        }
    }

    let domains = ftl_memory.domains()?;
    let strings = ftl_memory.strings()?;

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
    if params.audit.unwrap_or(false) {
        let audit_file = BufReader::new(env.read_file(PiholeFile::AuditLog)?);
        let audited_domains: Vec<String> =
            audit_file.lines().filter_map(|line| line.ok()).collect();

        // Get a vector of references to strings, to better compare with the domains
        let audited_domains: Vec<&str> = audited_domains.iter().map(String::as_str).collect();

        domains.retain(|domain| !audited_domains.contains(&domain.get_domain(&strings)));
    }

    // Sort the domains (descending by default)
    match (params.ascending.unwrap_or(false), blocked) {
        (false, false) => domains.sort_by(|a, b| {
            (b.query_count - b.blocked_count).cmp(&(a.query_count - a.blocked_count))
        }),
        (true, false) => domains.sort_by(|a, b| {
            (a.query_count - a.blocked_count).cmp(&(b.query_count - b.blocked_count))
        }),
        (false, true) => domains.sort_by(|a, b| b.blocked_count.cmp(&a.blocked_count)),
        (true, true) => domains.sort_by(|a, b| a.blocked_count.cmp(&b.blocked_count))
    }

    // Take into account the limit if specified
    if let Some(limit) = params.limit {
        if limit < domains.len() {
            domains.split_off(limit);
        }
    }

    // Map the domains into the output format
    let top_domains: Vec<Value> = domains
        .iter()
        .map(|domain| {
            let name = domain.get_domain(&strings);
            let count = if blocked {
                domain.blocked_count
            } else {
                domain.query_count - domain.blocked_count
            };

            json!({
                "domain": name,
                "count": count
            })
        })
        .collect();

    // Output format changes when getting top blocked domains
    if blocked {
        return reply_data(json!({
            "top_domains": top_domains,
            "blocked_queries": counters.blocked_queries
        }));
    } else {
        return reply_data(json!({
            "top_domains": top_domains,
            "total_queries": counters.total_queries
        }));
    }
}

#[cfg(test)]
mod test {
    use env::PiholeFile;
    use ftl::{FtlCounters, FtlDomain, FtlMemory, FtlRegexMatch};
    use std::collections::HashMap;
    use testing::TestBuilder;

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
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters {
                total_queries: 39,
                blocked_queries: 21,
                total_domains: 5,
                ..FtlCounters::default()
            }
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
