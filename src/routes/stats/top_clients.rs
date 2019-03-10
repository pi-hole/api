// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Clients Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::{FtlClient, FtlMemory},
    routes::{
        auth::User,
        stats::common::{remove_excluded_clients, remove_hidden_clients}
    },
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Error, Reply}
};
use rocket::{request::Form, State};

/// Get the top clients
#[get("/stats/top_clients?<params..>")]
pub fn top_clients(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: Form<TopClientParams>
) -> Reply {
    reply_data(get_top_clients(&ftl_memory, &env, params.into_inner())?)
}

/// Represents the possible GET parameters on `/stats/top_clients`
#[derive(FromForm)]
pub struct TopClientParams {
    limit: Option<usize>,
    inactive: Option<bool>,
    ascending: Option<bool>,
    blocked: Option<bool>
}

/// Represents the reply structure for top (blocked) clients
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TopClientsReply {
    top_clients: Vec<TopClientItemReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_queries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocked_queries: Option<usize>
}

/// Represents the reply structure for a top (blocked) client item
#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct TopClientItemReply {
    pub name: String,
    pub ip: String,
    pub count: usize
}

/// Get the top clients according to the parameters
fn get_top_clients(
    ftl_memory: &FtlMemory,
    env: &Env,
    params: TopClientParams
) -> Result<TopClientsReply, Error> {
    // Resolve the parameters
    let limit = params.limit.unwrap_or(10);
    let inactive = params.inactive.unwrap_or(false);
    let ascending = params.ascending.unwrap_or(false);
    let blocked = params.blocked.unwrap_or(false);

    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;

    // Check if the client details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)?
        >= FtlPrivacyLevel::HideDomainsAndClients
    {
        return if blocked {
            Ok(TopClientsReply {
                top_clients: Vec::new(),
                total_queries: None,
                blocked_queries: Some(counters.blocked_queries as usize)
            })
        } else {
            Ok(TopClientsReply {
                top_clients: Vec::new(),
                total_queries: Some(counters.total_queries as usize),
                blocked_queries: None
            })
        };
    }

    let strings = ftl_memory.strings(&lock)?;
    let clients = ftl_memory.clients(&lock)?;

    // Get an array of valid client references (FTL allocates more than it uses)
    let mut clients: Vec<&FtlClient> = clients
        .iter()
        .take(counters.total_clients as usize)
        .collect();

    // Ignore inactive clients by default (retain active clients)
    if !inactive {
        if blocked {
            clients.retain(|client| client.blocked_count > 0);
        } else {
            clients.retain(|client| client.query_count > 0);
        }
    }

    // Remove excluded and hidden clients
    remove_excluded_clients(&mut clients, env, &strings)?;
    remove_hidden_clients(&mut clients, &strings);

    // Sort the clients (descending by default)
    match (ascending, blocked) {
        (false, false) => clients.sort_by(|a, b| b.query_count.cmp(&a.query_count)),
        (true, false) => clients.sort_by(|a, b| a.query_count.cmp(&b.query_count)),
        (false, true) => clients.sort_by(|a, b| b.blocked_count.cmp(&a.blocked_count)),
        (true, true) => clients.sort_by(|a, b| a.blocked_count.cmp(&b.blocked_count))
    }

    // Take into account the limit
    if limit < clients.len() {
        clients.split_off(limit);
    }

    // Map the clients into the output format
    let top_clients: Vec<TopClientItemReply> = clients
        .into_iter()
        .map(|client| {
            let name = client.get_name(&strings).unwrap_or_default().to_owned();
            let ip = client.get_ip(&strings).to_owned();
            let count = if blocked {
                client.blocked_count
            } else {
                client.query_count
            } as usize;

            TopClientItemReply { name, ip, count }
        })
        .collect();

    // Output format changes when getting top blocked clients
    if blocked {
        Ok(TopClientsReply {
            top_clients,
            total_queries: None,
            blocked_queries: Some(counters.blocked_queries as usize)
        })
    } else {
        Ok(TopClientsReply {
            top_clients,
            total_queries: Some(counters.total_queries as usize),
            blocked_queries: None
        })
    }
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
                FtlClient::new(30, 10, 1, Some(2)),
                FtlClient::new(20, 5, 3, None),
                FtlClient::new(10, 0, 4, Some(5)),
                FtlClient::new(40, 0, 6, None),
                FtlClient::new(0, 0, 7, None),
                FtlClient::new(0, 0, 8, None),
            ],
            domains: Vec::new(),
            over_time: Vec::new(),
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters {
                total_queries: 100,
                blocked_queries: 15,
                total_clients: 6,
                ..FtlCounters::default()
            },
            settings: FtlSettings::default()
        }
    }

    /// The default behavior lists all active clients in descending order
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Show only active blocked clients (active in terms of blocked query
    /// count)
    #[test]
    fn blocked_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?blocked=true")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_clients": [
                    { "name": "client1", "ip": "10.1.1.1", "count": 10 },
                    { "name": "",        "ip": "10.1.1.2", "count": 5 }
                ],
                "blocked_queries": 15
            }))
            .test();
    }

    /// The number of clients shown is <= the limit
    #[test]
    fn limit() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?limit=2")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Same as the default behavior but in ascending order
    #[test]
    fn ascending() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?ascending=true")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_clients": [
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.4", "count": 40 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Privacy level 2 does not show any clients
    #[test]
    fn privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!({
                "top_clients": [],
                "total_queries": 100
            }))
            .test();
    }

    /// Privacy level 2 does not show any clients, and has a
    /// `"blocked_queries`" key instead of a `"total_queries"` key
    #[test]
    fn privacy_blocked() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?blocked=true")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!({
                "top_clients": [],
                "blocked_queries": 15
            }))
            .test();
    }

    /// Inactive clients are shown, but hidden clients are still not shown
    #[test]
    fn inactive_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?inactive=true")
            .ftl_memory(test_data())
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 },
                    { "name": "",        "ip": "10.1.1.5", "count":  0 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    /// Excluded clients are not shown
    #[test]
    fn excluded_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(
                PiholeFile::SetupVars,
                "API_EXCLUDE_CLIENTS=client3,10.1.1.2"
            )
            .expect_json(json!({
                "top_clients": [
                    { "name": "",        "ip": "10.1.1.4", "count": 40 },
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 }
                ],
                "total_queries": 100
            }))
            .test();
    }
}
