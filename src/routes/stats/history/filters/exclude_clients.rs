// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// SetupVars API_EXCLUDE_CLIENTS Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries,
    env::Env,
    ftl::{FtlMemory, FtlQuery, ShmLockGuard},
    settings::{ConfigEntry, SetupVarsEntry},
    util::Error
};
use diesel::{prelude::*, sqlite::Sqlite};
use std::collections::HashSet;

/// Apply the `SetupVarsEntry::ApiExcludeClients` setting
pub fn filter_excluded_clients<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    env: &Env,
    ftl_memory: &FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    // Get the excluded clients list
    let excluded_clients: Vec<String> = SetupVarsEntry::ApiExcludeClients
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();
    let excluded_clients: HashSet<&str> = excluded_clients.iter().map(String::as_str).collect();

    // End early if there are no excluded clients
    if excluded_clients.is_empty() {
        return Ok(queries_iter);
    }

    // Find the client IDs of the excluded clients
    let counters = ftl_memory.counters(ftl_lock)?;
    let strings = ftl_memory.strings(ftl_lock)?;
    let clients = ftl_memory.clients(ftl_lock)?;
    let excluded_client_ids: HashSet<usize> = clients
        .iter()
        .take(counters.total_clients as usize)
        .enumerate()
        .filter_map(|(i, client)| {
            let ip = client.get_ip(&strings);
            let name = client.get_name(&strings).unwrap_or_default();

            if excluded_clients.contains(ip) || excluded_clients.contains(name) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    // End if no clients match the excluded clients
    if excluded_client_ids.is_empty() {
        return Ok(queries_iter);
    }

    // Filter out the excluded domains using the domain IDs
    Ok(Box::new(queries_iter.filter(move |query| {
        !excluded_client_ids.contains(&(query.client_id as usize))
    })))
}

/// Apply the `SetupVarsEntry::ApiExcludeClients` setting to database queries
pub fn filter_excluded_clients_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    env: &Env
) -> Result<queries::BoxedQuery<'a, Sqlite>, Error> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    // Get the excluded clients list
    let excluded_clients: HashSet<String> = SetupVarsEntry::ApiExcludeClients
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();

    if excluded_clients.is_empty() {
        Ok(db_query)
    } else {
        Ok(db_query.filter(client.ne_all(excluded_clients)))
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_excluded_clients, filter_excluded_clients_db};
    use crate::{
        databases::ftl::connect_to_test_db,
        env::{Config, Env, PiholeFile},
        ftl::{FtlQuery, ShmLockGuard},
        routes::stats::history::{
            database::execute_query,
            testing::{test_memory, test_queries}
        },
        testing::TestEnvBuilder
    };
    use diesel::prelude::*;

    /// No queries should be filtered out if `API_EXCLUDE_CLIENTS` is empty
    #[test]
    fn clients_empty() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=")
                .build()
        );
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().collect();
        let filtered_queries: Vec<&FtlQuery> = filter_excluded_clients(
            Box::new(queries.iter()),
            &env,
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Queries with a client in the `API_EXCLUDE_CLIENTS` list should be
    /// removed
    #[test]
    fn clients() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=192.168.1.11")
                .build()
        );
        let queries = test_queries();
        let expected_queries = vec![
            &queries[0],
            &queries[1],
            &queries[2],
            &queries[6],
            &queries[7],
            &queries[8],
        ];
        let filtered_queries: Vec<&FtlQuery> = filter_excluded_clients(
            Box::new(queries.iter()),
            &env,
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Queries with a client in the `API_EXCLUDE_CLIENTS` list should be
    /// removed. This is a database filter.
    #[test]
    fn clients_db() {
        use crate::databases::ftl::queries::dsl::*;

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=127.0.0.1")
                .build()
        );

        let db_query = filter_excluded_clients_db(queries.into_boxed(), &env).unwrap();
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        assert_eq!(filtered_queries.len(), 1);
        assert_eq!(filtered_queries[0].client, "10.1.1.1".to_owned());
    }
}
