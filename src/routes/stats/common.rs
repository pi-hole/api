// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Code For Statistic Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::Env;
use ftl::{FtlClient, FtlStrings};
use settings::{ConfigEntry, SetupVarsEntry};
use util::Error;

/// Remove clients from the `clients` array if they show up in
/// [`SetupVarsEntry::ApiExcludeClients`].
///
/// [`SetupVarsEntry::ApiExcludeClients`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeClients
pub fn remove_excluded_clients(
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
mod tests {
    use super::remove_excluded_clients;
    use env::{Config, Env, PiholeFile};
    use ftl::{FtlClient, FtlCounters, FtlMemory};
    use std::collections::HashMap;
    use testing::TestEnvBuilder;

    /// Test data for `remove_excluded_clients`.
    fn test_data() -> FtlMemory {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "0.0.0.0".to_owned());

        FtlMemory::Test {
            clients: vec![
                FtlClient::new(30, 0, 1, Some(2)),
                FtlClient::new(20, 0, 3, None),
                FtlClient::new(0, 0, 4, None),
            ],
            strings,
            counters: FtlCounters::default()
        }
    }

    /// Only clients marked as excluded are removed
    #[test]
    fn only_removes_excluded() {
        let ftl_memory = test_data();

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(
                    PiholeFile::SetupVars,
                    "API_EXCLUDE_CLIENTS=10.1.1.2,client1"
                )
                .build()
        );

        let clients = ftl_memory.clients().unwrap();
        let mut clients = clients.iter().collect();
        remove_excluded_clients(&mut clients, &env, &ftl_memory.strings().unwrap()).unwrap();

        assert_eq!(clients, vec![&FtlClient::new(0, 0, 4, None)]);
    }

    /// When there are no excluded clients, the vector is not modified
    #[test]
    fn unmodified_when_not_excluded() {
        let ftl_memory = test_data();

        let env = Env::Test(Config::default(), TestEnvBuilder::new().build());

        let clients = ftl_memory.clients().unwrap();
        let mut clients: Vec<&FtlClient> = clients.iter().collect();
        let clients_clone = clients.clone();
        remove_excluded_clients(&mut clients, &env, &ftl_memory.strings().unwrap()).unwrap();

        assert_eq!(clients, clients_clone);
    }
}
