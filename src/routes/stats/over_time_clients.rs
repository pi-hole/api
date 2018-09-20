// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Clients Over Time Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use env::Env;
use ftl::{FtlClient, FtlMemory};
use rocket::State;
use rocket_contrib::Value;
use routes::stats::common::{remove_excluded_clients, remove_hidden_clients};
use settings::{ConfigEntry, FtlConfEntry, FtlPrivacyLevel};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use util::{reply_data, Reply};

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
    let counters = ftl_memory.counters()?;
    let strings = ftl_memory.strings()?;
    let over_time = ftl_memory.over_time()?;
    let clients = ftl_memory.clients()?;

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

    // Get the client overTime data for each client.
    // This is done without an iterator because `ftl_memory.over_time_client` could
    // throw an error, which should be returned.
    let client_data = {
        let mut client_data = Vec::with_capacity(clients.len());

        for &client in &clients {
            // Client overTime data is stored using the client ID (index)
            client_data.push(ftl_memory.over_time_client(client_ids[client])?);
        }

        client_data
    };

    // Get the current timestamp, to be used when getting overTime data
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time web backwards")
        .as_secs() as f64;

    // Get the max log age FTL setting, to be used when getting overTime data
    let max_log_age = FtlConfEntry::MaxLogAge.read_as::<f64>(&env).unwrap_or(24.0) * 3600.0;

    // Get the valid over time slots (starting at the max-log-age timestamp).
    // Then, combine with the client data from above to get the final overTime
    // output.
    let over_time: Vec<Value> = over_time
        .iter()
        .take(counters.over_time_size as usize)
        .enumerate()
        // Skip the overTime slots without any data, and any slots which are
        // before the max-log-age time.
        .skip_while(|(_, time)| {
            (time.total_queries <= 0 && time.blocked_queries <= 0)
                || ((time.timestamp as f64) < timestamp - max_log_age)
        })
        .map(|(i, time)| {
            // Get the client data for this time slot
            let data: Vec<usize> = client_data
                .iter()
                // Each client data is indexed according to the overTime index
                .map(|client| *client.get(i).unwrap_or(&0) as usize)
                .collect();

            json!({
                "timestamp": time.timestamp,
                "data": data
            })
        })
        .collect();

    // Convert clients into the output format
    let clients: Vec<Value> = clients
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
    use env::PiholeFile;
    use ftl::{FtlClient, FtlCounters, FtlMemory, FtlOverTime};
    use std::collections::HashMap;
    use testing::TestBuilder;

    /// There are 6 clients, two inactive, one hidden, and two with names.
    /// There are 3 overTime slots, with cooresponding client overTime data
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
            over_time: vec![
                FtlOverTime::new(0, 0, 0, 0, 0, [0; 7]),
                FtlOverTime::new(1, 3, 0, 2, 1, [0; 7]),
                FtlOverTime::new(2, 2, 2, 0, 0, [0; 7]),
                FtlOverTime::new(3, 1, 1, 1, 0, [0; 7]),
            ],
            over_time_clients: vec![
                vec![0, 1, 0, 0],
                vec![0, 1, 0, 0],
                vec![0, 1, 0, 0],
                vec![0, 0, 1, 0],
                vec![0, 0, 1, 0],
                vec![0, 0, 0, 1],
            ],
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters {
                total_clients: 6,
                over_time_size: 4,
                ..FtlCounters::default()
            }
        }
    }

    /// Default params will show overTime data from all non-hidden and
    /// non-excluded clients, and will skip overTime slots with no queries.
    #[test]
    fn default_params() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/clients")
            .ftl_memory(test_data())
            // Abuse From<&str> for f64 and use all overTime data
            .file(PiholeFile::FtlConfig, "MAXLOGAGE=inf")
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

    /// Only overTime slots within the MAXLOGAGE value are considered
    #[test]
    fn max_log_age() {
        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/clients")
            .ftl_memory(test_data())
            // Abuse From<&str> for f64 and use all overTime data
            .file(PiholeFile::FtlConfig, "MAXLOGAGE=0")
            .expect_json(json!({
                "clients": [
                    { "name": "client1", "ip": "10.1.1.1" },
                    { "name": "",        "ip": "10.1.1.2" },
                    { "name": "client3", "ip": "10.1.1.3" },
                    { "name": "",        "ip": "10.1.1.4" },
                    { "name": "",        "ip": "10.1.1.5" }
                ],
                "over_time": []
            }))
            .test();
    }
}
