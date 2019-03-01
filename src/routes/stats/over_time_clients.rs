// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Clients Over Time Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::{FtlClient, FtlMemory},
    routes::{
        auth::User,
        stats::common::{
            get_current_over_time_slot, remove_excluded_clients, remove_hidden_clients
        }
    },
    settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel},
    util::{reply_data, Reply}
};
use rocket::State;
use rocket_contrib::json::JsonValue;
use std::collections::HashMap;

/// Get the client queries over time
#[get("/stats/overTime/clients")]
pub fn over_time_clients(_auth: User, ftl_memory: State<FtlMemory>, env: State<Env>) -> Reply {
    // Check if client details are private
    if FtlConfEntry::PrivacyLevel.read_as::<FtlPrivacyLevel>(&env)?
        >= FtlPrivacyLevel::HideDomainsAndClients
    {
        return reply_data(json!({
            "over_time": [],
            "clients": []
        }));
    }

    // Load FTL shared memory
    let lock = ftl_memory.lock()?;
    let counters = ftl_memory.counters(&lock)?;
    let strings = ftl_memory.strings(&lock)?;
    let over_time = ftl_memory.over_time(&lock)?;
    let clients = ftl_memory.clients(&lock)?;

    // Store the client IDs (indexes), even after going through filters
    let mut client_ids = HashMap::new();

    // Get the clients we will be returning overTime data of
    let mut clients: Vec<&FtlClient> = clients
        .iter()
        .take(counters.total_clients as usize)
        // Enumerate so the client ID can be stored
        .enumerate()
        .map(|(i, client)| {
            client_ids.insert(client, i);

            // After this, we don't need the index in this iterator
            client
        })
        .collect();

    // Ignore hidden and excluded clients
    remove_hidden_clients(&mut clients, &strings);
    remove_excluded_clients(&mut clients, &env, &strings)?;

    // Get the valid over time slots (Skip while the slots are empty).
    // Then, combine with the client overTime data to get the final overTime
    // output.
    let over_time: Vec<JsonValue> = over_time
        .iter()
        // Take all of the slots including the current slot
        .take(get_current_over_time_slot(&over_time) + 1)
        .enumerate()
        // Skip the overTime slots without any data
        .skip_while(|(_, time)| {
            (time.total_queries <= 0 && time.blocked_queries <= 0)
        })
        .map(|(i, time)| {
            // Get the client data for this time slot
            let data: Vec<usize> = clients
                .iter()
                // Each client data is indexed according to the overTime index
                .map(|client| *client.over_time.get(i).unwrap_or(&0) as usize)
                .collect();

            json!({
                "timestamp": time.timestamp,
                "data": data
            })
        })
        .collect();

    // Convert clients into the output format
    let clients: Vec<JsonValue> = clients
        .into_iter()
        .map(|client| {
            let name = client.get_name(&strings).unwrap_or_default();
            let ip = client.get_ip(&strings);

            json!({
                "name": name,
                "ip": ip
            })
        })
        .collect();

    reply_data(json!({
        "over_time": over_time,
        "clients": clients
    }))
}

#[cfg(test)]
mod test {
    use crate::{
        env::PiholeFile,
        ftl::{FtlClient, FtlCounters, FtlMemory, FtlOverTime, FtlSettings},
        testing::TestBuilder
    };
    use std::collections::HashMap;

    /// There are 6 clients, two inactive, one hidden, and two with names.
    /// There are 3 overTime slots, with corresponding client overTime data
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
                FtlClient::new(1, 0, 1, Some(2)).with_over_time(vec![0, 1, 0, 0]),
                FtlClient::new(1, 0, 3, None).with_over_time(vec![0, 1, 0, 0]),
                FtlClient::new(1, 0, 4, Some(5)).with_over_time(vec![0, 1, 0, 0]),
                FtlClient::new(1, 0, 6, None).with_over_time(vec![0, 0, 1, 0]),
                FtlClient::new(0, 0, 7, None).with_over_time(vec![0, 0, 1, 0]),
                FtlClient::new(0, 0, 8, None).with_over_time(vec![0, 0, 0, 1]),
            ],
            domains: Vec::new(),
            over_time: vec![
                FtlOverTime::new(0, 0, 0, 0, 0, [0; 7]),
                FtlOverTime::new(1, 3, 0, 2, 1, [0; 7]),
                FtlOverTime::new(2, 2, 2, 0, 0, [0; 7]),
                FtlOverTime::new(3, 1, 1, 1, 0, [0; 7]),
            ],
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

    /// Default params will show overTime data from all non-hidden and
    /// non-excluded clients, and will skip overTime slots with no queries.
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/clients")
            .ftl_memory(test_data())
            .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=client1")
            .expect_json(json!({
                "clients": [
                    { "name": "",        "ip": "10.1.1.2" },
                    { "name": "client3", "ip": "10.1.1.3" },
                    { "name": "",        "ip": "10.1.1.4" },
                    { "name": "",        "ip": "10.1.1.5" }
                ],
                "over_time": [
                    { "timestamp": 1, "data": [1, 1, 0, 0] },
                    { "timestamp": 2, "data": [0, 0, 1, 1] },
                    { "timestamp": 3, "data": [0, 0, 0, 0] },
                ]
            }))
            .test();
    }
}
