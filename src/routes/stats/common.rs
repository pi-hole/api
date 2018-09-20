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
use ftl::{FtlClient, FtlDomain, FtlStrings};
use settings::{ConfigEntry, SetupVarsEntry};
use util::Error;

/// Remove clients from the `clients` vector if they show up in
/// [`SetupVarsEntry::ApiExcludeClients`].
///
/// [`SetupVarsEntry::ApiExcludeClients`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeClients
pub fn remove_excluded_clients(
    clients: &mut Vec<&FtlClient>,
    env: &Env,
    strings: &FtlStrings
) -> Result<(), Error> {
    let excluded_clients = SetupVarsEntry::ApiExcludeClients.read(env)?.to_lowercase();
    let excluded_clients: Vec<&str> = excluded_clients
        .split(",")
        .filter(|s| !s.is_empty())
        .collect();

    if !excluded_clients.is_empty() {
        // Only retain clients which do not appear in the exclusion list
        clients.retain(|client| {
            let ip = client.get_ip(strings);
            let name = client.get_name(strings).unwrap_or_default().to_lowercase();

            !excluded_clients.contains(&ip) && !excluded_clients.contains(&name.as_str())
        })
    }

    Ok(())
}

/// Remove domains from the `domains` vector if they show up in
/// [`SetupVarsEntry::ApiExcludeDomains`].
///
/// [`SetupVarsEntry::ApiExcludeDomains`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeDomains
pub fn remove_excluded_domains(
    domains: &mut Vec<&FtlDomain>,
    env: &Env,
    strings: &FtlStrings
) -> Result<(), Error> {
    let excluded_domains = SetupVarsEntry::ApiExcludeDomains.read(env)?.to_lowercase();
    let excluded_domains: Vec<&str> = excluded_domains
        .split(",")
        .filter(|s| !s.is_empty())
        .collect();

    if !excluded_domains.is_empty() {
        // Only retain domains which do not appear in the exclusion list
        domains.retain(|domain| !excluded_domains.contains(&domain.get_domain(strings)));
    }

    Ok(())
}

/// Remove clients from the `clients` vector if they are marked as hidden due
/// to the privacy level.
pub fn remove_hidden_clients(clients: &mut Vec<&FtlClient>, strings: &FtlStrings) {
    clients.retain(|client| client.get_ip(strings) != "0.0.0.0");
}

/// Remove domains from the `domains` vector if they are marked as hidden due
/// to the privacy level.
pub fn remove_hidden_domains(domains: &mut Vec<&FtlDomain>, strings: &FtlStrings) {
    domains.retain(|domain| domain.get_domain(strings) != "hidden");
}

#[cfg(test)]
mod tests {
    use super::{
        remove_excluded_clients, remove_excluded_domains, remove_hidden_clients,
        remove_hidden_domains
    };
    use env::{Config, Env, PiholeFile};
    use ftl::{FtlClient, FtlCounters, FtlDomain, FtlMemory, FtlRegexMatch};
    use std::collections::HashMap;
    use testing::TestEnvBuilder;

    /// There are 4 clients, one hidden
    fn test_data() -> FtlMemory {
        let mut strings = HashMap::new();
        strings.insert(1, "10.1.1.1".to_owned());
        strings.insert(2, "client1".to_owned());
        strings.insert(3, "10.1.1.2".to_owned());
        strings.insert(4, "0.0.0.0".to_owned());
        strings.insert(5, "example.com".to_owned());
        strings.insert(6, "example.net".to_owned());
        strings.insert(7, "hidden".to_owned());

        FtlMemory::Test {
            clients: vec![
                FtlClient::new(30, 0, 1, Some(2)),
                FtlClient::new(20, 0, 3, None),
                FtlClient::new(0, 0, 4, None),
            ],
            domains: vec![
                FtlDomain::new(0, 0, 5, FtlRegexMatch::Unknown),
                FtlDomain::new(0, 0, 6, FtlRegexMatch::Unknown),
                FtlDomain::new(0, 0, 7, FtlRegexMatch::Unknown),
            ],
            over_time: Vec::new(),
            over_time_clients: Vec::new(),
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters::default()
        }
    }

    /// Clients marked as excluded are removed
    #[test]
    fn excluded_clients() {
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

    /// Domains marked as excluded are removed
    #[test]
    fn excluded_domains() {
        let ftl_memory = test_data();

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(
                    PiholeFile::SetupVars,
                    "API_EXCLUDE_DOMAINS=google.com,example.com"
                )
                .build()
        );

        let domains = ftl_memory.domains().unwrap();
        let mut clients = domains.iter().collect();

        remove_excluded_domains(&mut clients, &env, &ftl_memory.strings().unwrap()).unwrap();

        assert_eq!(
            clients,
            vec![
                &FtlDomain::new(0, 0, 6, FtlRegexMatch::Unknown),
                &FtlDomain::new(0, 0, 7, FtlRegexMatch::Unknown),
            ]
        );
    }

    /// Clients marked as hidden are removed
    #[test]
    fn hidden_clients() {
        let ftl_memory = test_data();

        let clients = ftl_memory.clients().unwrap();
        let mut clients: Vec<&FtlClient> = clients.iter().collect();
        let mut clients_clone = clients.clone();
        clients_clone.remove(2);

        remove_hidden_clients(&mut clients, &ftl_memory.strings().unwrap());

        assert_eq!(clients, clients_clone);
    }

    /// Domains marked as hidden are removed
    #[test]
    fn hidden_domains() {
        let ftl_memory = test_data();

        let domains = ftl_memory.domains().unwrap();
        let mut domains: Vec<&FtlDomain> = domains.iter().collect();
        let mut domains_clone = domains.clone();
        domains_clone.remove(2);

        remove_hidden_domains(&mut domains, &ftl_memory.strings().unwrap());

        assert_eq!(domains, domains_clone);
    }
}
