// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Clients Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use ftl::{FtlClient, FtlMemory, FtlPrivacyLevel};
use rocket::State;
use rocket_contrib::Value;
use routes::stats::common::{remove_excluded_clients, remove_hidden_clients};
use settings::{ConfigEntry, FtlConfEntry};
use util::{reply_data, Reply};

/// Get the top clients with default parameters
#[get("/stats/top_clients")]
pub fn top_clients(_auth: User, ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    get_top_clients(&ftl_memory, &env, TopClientParams::default())
}

/// Get the top clients with specified parameters
#[get("/stats/top_clients?<params>")]
pub fn top_clients_params(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: TopClientParams
) -> Reply {
    get_top_clients(&ftl_memory, &env, params)
}

/// Represents the possible GET parameters on `/stats/top_clients`
#[derive(FromForm)]
pub struct TopClientParams {
    limit: Option<usize>,
    inactive: Option<bool>,
    ascending: Option<bool>
}

impl Default for TopClientParams {
    fn default() -> Self {
        TopClientParams {
            limit: None,
            inactive: Some(false),
            ascending: Some(false)
        }
    }
}

/// Get the top clients according to the parameters
fn get_top_clients(ftl_memory: &FtlMemory, env: &Env, params: TopClientParams) -> Reply {
    let counters = ftl_memory.counters()?;

    // Check if the client details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)? >= FtlPrivacyLevel::Maximum {
        return reply_data(json!({
            "top_clients": [],
            "total_queries": counters.total_queries
        }));
    }

    let strings = ftl_memory.strings()?;
    let clients = ftl_memory.clients()?;

    // Get an array of valid client references (FTL allocates more than it uses)
    let mut clients: Vec<&FtlClient> = clients
        .iter()
        .take(counters.total_clients as usize)
        .collect();

    // Sort the clients (descending by default)
    if params.ascending.unwrap_or(false) {
        clients.sort_by(|a, b| a.query_count.cmp(&b.query_count));
    } else {
        clients.sort_by(|a, b| b.query_count.cmp(&a.query_count));
    }

    // Ignore inactive clients by default (retain active clients)
    if !params.inactive.unwrap_or(false) {
        clients.retain(|client| client.query_count > 0);
    }

    // Ignore excluded clients
    remove_excluded_clients(&mut clients, env, &strings)?;

    // Ignore hidden clients (due to privacy level)
    remove_hidden_clients(&mut clients, &strings);

    // Take into account the limit if specified
    if let Some(limit) = params.limit {
        if limit < clients.len() {
            clients.split_off(limit);
        }
    }

    // Map the clients into the output format
    let top_clients: Vec<Value> = clients
        .into_iter()
        .map(|client| {
            let name = client.get_name(&strings).unwrap_or_default();
            let ip = client.get_ip(&strings);
            let count = client.query_count;

            json!({
                "name": name,
                "ip": ip,
                "count": count
            })
        })
        .collect();

    reply_data(json!({
        "top_clients": top_clients,
        "total_queries": counters.total_queries
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
                FtlClient::new(30, 0, 1, Some(2)),
                FtlClient::new(20, 0, 3, None),
                FtlClient::new(10, 0, 4, Some(5)),
                FtlClient::new(40, 0, 6, None),
                FtlClient::new(0, 0, 7, None),
                FtlClient::new(0, 0, 8, None),
            ],
            strings,
            counters: FtlCounters {
                total_queries: 100,
                total_clients: 6,
                ..FtlCounters::default()
            }
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

    /// Maximum privacy level does not show any clients
    #[test]
    fn max_privacy() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=3")
            .expect_json(json!({
                "top_clients": [],
                "total_queries": 100
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
