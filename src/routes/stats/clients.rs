// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Clients Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::FtlMemory,
    routes::{
        auth::User,
        stats::common::{remove_excluded_clients, remove_hidden_clients}
    },
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Reply}
};
use rocket::{request::Form, State};
use rocket_contrib::json::JsonValue;

/// Get the client information with default parameters
#[get("/stats/clients")]
pub fn clients(_auth: User, ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    get_clients(&ftl_memory, &env, ClientParams::default())
}

/// Get the client information with specified parameters
#[get("/stats/clients?<params..>")]
pub fn clients_params(
    _auth: User,
    ftl_memory: State<FtlMemory>,
    env: State<Env>,
    params: Form<ClientParams>
) -> Reply {
    get_clients(&ftl_memory, &env, params.into_inner())
}

/// The possible GET parameters for `/stats/clients`
#[derive(FromForm)]
pub struct ClientParams {
    inactive: Option<bool>
}

impl Default for ClientParams {
    fn default() -> Self {
        ClientParams {
            inactive: Some(false)
        }
    }
}

/// Get client info according to the parameters
pub fn get_clients(ftl_memory: &FtlMemory, env: &Env, params: ClientParams) -> Reply {
    // Check if client details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)?
        >= FtlPrivacyLevel::HideDomainsAndClients
    {
        return reply_data([0; 0]);
    }

    let lock = ftl_memory.lock()?;
    let strings = ftl_memory.strings(&lock)?;
    let counters = ftl_memory.counters(&lock)?;
    let clients = ftl_memory.clients(&lock)?;

    // Get an array of valid client references (FTL allocates more than it uses)
    let mut clients = clients
        .iter()
        .take(counters.total_clients as usize)
        .collect();

    // Ignore excluded clients
    remove_excluded_clients(&mut clients, &env, &strings)?;

    // Ignore hidden clients (due to privacy level)
    remove_hidden_clients(&mut clients, &strings);

    // Ignore inactive clients by default (retain active clients)
    if !params.inactive.unwrap_or(false) {
        clients.retain(|client| client.query_count > 0);
    }

    // Only output the name and IP of each client
    reply_data(
        clients
            .iter()
            .map(|client| {
                let name = client.get_name(&strings).unwrap_or_default();
                let ip = client.get_ip(&strings);

                json!({
                    "name": name,
                    "ip": ip
                })
            })
            .collect::<Vec<JsonValue>>()
    )
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
                total_clients: 6,
                ..FtlCounters::default()
            },
            settings: FtlSettings::default()
        }
    }

    /// The default behavior lists all active clients
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/clients")
            .ftl_memory(test_data())
            .expect_json(json!([
                { "name": "client1", "ip": "10.1.1.1" },
                { "name": "",        "ip": "10.1.1.2" },
                { "name": "client3", "ip": "10.1.1.3" },
                { "name": "",        "ip": "10.1.1.4" }
            ]))
            .test();
    }

    /// Privacy level 2 does not show any clients
    #[test]
    fn privacy_hides_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/clients")
            .ftl_memory(test_data())
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .expect_json(json!([]))
            .test();
    }

    /// Inactive clients are shown, but hidden clients are still not shown
    #[test]
    fn inactive_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/clients?inactive=true")
            .ftl_memory(test_data())
            .expect_json(json!([
                { "name": "client1", "ip": "10.1.1.1" },
                { "name": "",        "ip": "10.1.1.2" },
                { "name": "client3", "ip": "10.1.1.3" },
                { "name": "",        "ip": "10.1.1.4" },
                { "name": "",        "ip": "10.1.1.5" }
            ]))
            .test();
    }

    /// Excluded clients are not shown
    #[test]
    fn excluded_clients() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/clients")
            .ftl_memory(test_data())
            .file(
                PiholeFile::SetupVars,
                "API_EXCLUDE_CLIENTS=client3,10.1.1.2"
            )
            .expect_json(json!([
                { "name": "client1", "ip": "10.1.1.1" },
                { "name": "",        "ip": "10.1.1.4" }
            ]))
            .test();
    }
}
