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
use ftl::{FtlClient, FtlMemory, FtlPrivacyLevel, FtlStrings};
use rocket::State;
use rocket_contrib::Value;
use settings::{ConfigEntry, FtlConfEntry, SetupVarsEntry};
use util::{reply_data, Error, Reply};

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
    /// The default parameters of top_clients requests
    fn default() -> Self {
        TopClientParams {
            limit: None,
            inactive: Some(false),
            ascending: Some(false)
        }
    }
}

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
        clients.sort_by(|a, b| a.query_count().cmp(&b.query_count()));
    } else {
        clients.sort_by(|a, b| b.query_count().cmp(&a.query_count()));
    }

    // Ignore inactive clients by default (retain active clients)
    if !params.inactive.unwrap_or(false) {
        clients.retain(|client| client.query_count() > 0);
    }

    // Ignore excluded clients
    remove_excluded_clients(&mut clients, env, &strings)?;

    // Ignore hidden clients (due to privacy level)
    clients.retain(|client| strings.get_str(client.ip_str_id()).unwrap_or_default() != "0.0.0.0");

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
            let name = strings
                .get_str(client.name_str_id().unwrap_or(0))
                .unwrap_or_default();
            let ip = strings
                .get_str(client.ip_str_id())
                .unwrap_or_default()
                .to_owned();
            let count = client.query_count();

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

/// Remove clients from the `clients` array if they show up in
/// [`SetupVarsEntry::ApiExcludeClients`].
///
/// [`SetupVarsEntry::ApiExcludeClients`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeClients
fn remove_excluded_clients(
    clients: &mut Vec<&FtlClient>,
    env: &Env,
    strings: &FtlStrings
) -> Result<(), Error> {
    let excluded_clients_array = SetupVarsEntry::ApiExcludeClients.read(env)?.to_lowercase();
    let excluded_clients: Vec<&str> = excluded_clients_array
        .split(",")
        .filter(|s| !s.is_empty())
        .collect();

    if !excluded_clients.is_empty() {
        // Only retain clients which do not appear in the exclusion list
        clients.retain(|client| {
            let ip = strings.get_str(client.ip_str_id()).unwrap_or_default();
            let name = strings
                .get_str(client.name_str_id().unwrap_or_default())
                .unwrap_or_default()
                .to_lowercase();

            !excluded_clients.contains(&ip) && !excluded_clients.contains(&name.as_str())
        })
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use ftl::{FtlClient, FtlCounters, FtlMemory};
    use rmp::encode;
    use std::collections::HashMap;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_top_clients() {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "10.1.1.3".to_owned());
        strings.insert(5, "client3".to_owned());

        let ftl_memory = FtlMemory::Test {
            clients: vec![
                FtlClient::new(30, 0, 1, Some(2)),
                FtlClient::new(20, 0, 3, None),
                FtlClient::new(10, 0, 4, Some(5)),
            ],
            strings,
            counters: FtlCounters {
                total_queries: 100,
                total_clients: 3,
                ..FtlCounters::default()
            }
        };

        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl_memory(ftl_memory)
            .expect_json(json!({
                "top_clients": [
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 },
                    { "name": "client3", "ip": "10.1.1.3", "count": 10 }
                ],
                "total_queries": 100
            }))
            .test();
    }

    #[test]
    fn test_top_clients_limit() {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "10.1.1.3".to_owned());
        strings.insert(5, "client3".to_owned());

        let ftl_memory = FtlMemory::Test {
            clients: vec![
                FtlClient::new(30, 0, 1, Some(2)),
                FtlClient::new(20, 0, 3, None),
                FtlClient::new(10, 0, 4, Some(5)),
            ],
            strings,
            counters: FtlCounters {
                total_queries: 100,
                total_clients: 3,
                ..FtlCounters::default()
            }
        };

        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?limit=2")
            .ftl_memory(ftl_memory)
            .expect_json(json!({
                "top_clients": [
                    { "name": "client1", "ip": "10.1.1.1", "count": 30 },
                    { "name": "",        "ip": "10.1.1.2", "count": 20 }
                ],
                "total_queries": 100
            }))
            .test();
    }
}
