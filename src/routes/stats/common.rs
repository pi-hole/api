// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Code For Statistic Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    ftl::{FtlClient, FtlDomain, FtlOverTime, FtlStrings, OVERTIME_SLOTS},
    settings::{ConfigEntry, SetupVarsEntry},
    util::Error
};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH}
};

/// The designated hidden domain
pub const HIDDEN_DOMAIN: &str = "hidden";

/// The designated hidden client IP address
pub const HIDDEN_CLIENT: &str = "0.0.0.0";

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
    let excluded_clients = get_excluded_clients(env)?;
    let excluded_clients: HashSet<&str> = excluded_clients.iter().map(String::as_str).collect();

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

/// Get the clients from [`SetupVarsEntry::ApiExcludeClients`] in lowercase.
///
/// [`SetupVarsEntry::ApiExcludeClients`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeClients
pub fn get_excluded_clients(env: &Env) -> Result<Vec<String>, Error> {
    Ok(SetupVarsEntry::ApiExcludeClients
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect())
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
    let excluded_domains: Vec<String> = get_excluded_domains(env)?;
    let excluded_domains: HashSet<&str> = excluded_domains.iter().map(String::as_str).collect();

    if !excluded_domains.is_empty() {
        // Only retain domains which do not appear in the exclusion list
        domains.retain(|domain| !excluded_domains.contains(&domain.get_domain(strings)));
    }

    Ok(())
}

/// Get the domains from [`SetupVarsEntry::ApiExcludeDomains`] in lowercase.
///
/// [`SetupVarsEntry::ApiExcludeDomains`]:
/// ../../../settings/entries/enum.SetupVarsEntry.html#variant.ApiExcludeDomains
pub fn get_excluded_domains(env: &Env) -> Result<Vec<String>, Error> {
    Ok(SetupVarsEntry::ApiExcludeDomains
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect())
}

/// Remove clients from the `clients` vector if they are marked as hidden due
/// to the privacy level.
pub fn remove_hidden_clients(clients: &mut Vec<&FtlClient>, strings: &FtlStrings) {
    clients.retain(|client| client.get_ip(strings) != HIDDEN_CLIENT);
}

/// Remove domains from the `domains` vector if they are marked as hidden due
/// to the privacy level.
pub fn remove_hidden_domains(domains: &mut Vec<&FtlDomain>, strings: &FtlStrings) {
    domains.retain(|domain| domain.get_domain(strings) != HIDDEN_DOMAIN);
}

/// Get the current overTime slot index, based on the current time. If all of
/// the slots are in the past, then the last slot index will be returned.
pub fn get_current_over_time_slot(over_time: &[FtlOverTime]) -> usize {
    // Get the current timestamp so we can ignore future overTime slots
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time web backwards")
        .as_secs();

    // Find the current slot
    over_time
        .iter()
        .enumerate()
        .find_map(|(i, item)| {
            if item.timestamp as u64 >= current_timestamp {
                Some(i)
            } else {
                None
            }
        })
        .unwrap_or(OVERTIME_SLOTS - 1)
}

#[cfg(test)]
mod tests {
    use super::{
        remove_excluded_clients, remove_excluded_domains, remove_hidden_clients,
        remove_hidden_domains
    };
    use crate::{
        env::PiholeFile,
        ftl::{
            FtlClient, FtlCounters, FtlDomain, FtlMemory, FtlRegexMatch, FtlSettings, ShmLockGuard
        },
        testing::TestEnvBuilder
    };
    use std::collections::HashMap;

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
            strings,
            upstreams: Vec::new(),
            queries: Vec::new(),
            counters: FtlCounters::default(),
            settings: FtlSettings::default()
        }
    }

    /// Clients marked as excluded are removed
    #[test]
    fn excluded_clients() {
        let ftl_memory = test_data();

        let env = TestEnvBuilder::new()
            .file(
                PiholeFile::SetupVars,
                "API_EXCLUDE_CLIENTS=10.1.1.2,client1"
            )
            .build();

        let lock_guard = ShmLockGuard::Test;
        let clients = ftl_memory.clients(&lock_guard).unwrap();
        let mut clients = clients.iter().collect();

        remove_excluded_clients(
            &mut clients,
            &env,
            &ftl_memory.strings(&lock_guard).unwrap()
        )
        .unwrap();

        assert_eq!(clients, vec![&FtlClient::new(0, 0, 4, None)]);
    }

    /// Domains marked as excluded are removed
    #[test]
    fn excluded_domains() {
        let ftl_memory = test_data();

        let env = TestEnvBuilder::new()
            .file(
                PiholeFile::SetupVars,
                "API_EXCLUDE_DOMAINS=google.com,example.com"
            )
            .build();

        let lock_guard = ShmLockGuard::Test;
        let domains = ftl_memory.domains(&lock_guard).unwrap();
        let mut clients = domains.iter().collect();

        remove_excluded_domains(
            &mut clients,
            &env,
            &ftl_memory.strings(&lock_guard).unwrap()
        )
        .unwrap();

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
        let lock_guard = ShmLockGuard::Test;

        let clients = ftl_memory.clients(&lock_guard).unwrap();
        let mut clients: Vec<&FtlClient> = clients.iter().collect();
        let mut clients_clone = clients.clone();
        clients_clone.remove(2);

        remove_hidden_clients(&mut clients, &ftl_memory.strings(&lock_guard).unwrap());

        assert_eq!(clients, clients_clone);
    }

    /// Domains marked as hidden are removed
    #[test]
    fn hidden_domains() {
        let ftl_memory = test_data();
        let lock_guard = ShmLockGuard::Test;

        let domains = ftl_memory.domains(&lock_guard).unwrap();
        let mut domains: Vec<&FtlDomain> = domains.iter().collect();
        let mut domains_clone = domains.clone();
        domains_clone.remove(2);

        remove_hidden_domains(&mut domains, &ftl_memory.strings(&lock_guard).unwrap());

        assert_eq!(domains, domains_clone);
    }
}
