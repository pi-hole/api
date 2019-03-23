// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// SetupVars API_EXCLUDE_DOMAINS Filter
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

/// Apply the `SetupVarsEntry::ApiExcludeDomains` setting
pub fn filter_excluded_domains<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    env: &Env,
    ftl_memory: &FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    // Get the excluded domains list
    let excluded_domains: Vec<String> = SetupVarsEntry::ApiExcludeDomains
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();
    let excluded_domains: HashSet<&str> = excluded_domains.iter().map(String::as_str).collect();

    // End early if there are no excluded domains
    if excluded_domains.is_empty() {
        return Ok(queries_iter);
    }

    // Find the domain IDs of the excluded domains
    let counters = ftl_memory.counters(ftl_lock)?;
    let strings = ftl_memory.strings(ftl_lock)?;
    let domains = ftl_memory.domains(ftl_lock)?;
    let excluded_domain_ids: HashSet<usize> = domains
        .iter()
        .take(counters.total_domains as usize)
        .enumerate()
        .filter_map(|(i, domain)| {
            if excluded_domains.contains(domain.get_domain(&strings)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    // End if no domains match the excluded domains
    if excluded_domain_ids.is_empty() {
        return Ok(queries_iter);
    }

    // Filter out the excluded domains using the domain IDs
    Ok(Box::new(queries_iter.filter(move |query| {
        !excluded_domain_ids.contains(&(query.domain_id as usize))
    })))
}

/// Apply the `SetupVarsEntry::ApiExcludeDomains` setting to database queries
pub fn filter_excluded_domains_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    env: &Env
) -> Result<queries::BoxedQuery<'a, Sqlite>, Error> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    // Get the excluded domains list
    let excluded_domains: HashSet<String> = SetupVarsEntry::ApiExcludeDomains
        .read_list(env)?
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();

    if excluded_domains.is_empty() {
        Ok(db_query)
    } else {
        Ok(db_query.filter(domain.ne_all(excluded_domains)))
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_excluded_domains, filter_excluded_domains_db};
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

    /// No queries should be filtered out if `API_EXCLUDE_DOMAINS` is empty
    #[test]
    fn domains_empty() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_EXCLUDE_DOMAINS=")
                .build()
        );
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> = queries.iter().collect();
        let filtered_queries: Vec<&FtlQuery> = filter_excluded_domains(
            Box::new(queries.iter()),
            &env,
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Queries with a domain in the `API_EXCLUDE_DOMAINS` list should be
    /// removed
    #[test]
    fn domains() {
        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(PiholeFile::SetupVars, "API_EXCLUDE_DOMAINS=domain2.com")
                .build()
        );
        let queries = test_queries();
        let expected_queries: Vec<&FtlQuery> =
            queries.iter().filter(|query| query.id != 4).collect();
        let filtered_queries: Vec<&FtlQuery> = filter_excluded_domains(
            Box::new(queries.iter()),
            &env,
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Queries with a domain in the `API_EXCLUDE_DOMAIN` list should be
    /// removed. This is a database filter.
    #[test]
    fn domains_db() {
        use crate::databases::ftl::queries::dsl::*;

        let env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(
                    PiholeFile::SetupVars,
                    "API_EXCLUDE_DOMAINS=0.ubuntu.pool.ntp.org,1.ubuntu.pool.ntp.org"
                )
                .build()
        );

        let db_query = filter_excluded_domains_db(queries.into_boxed(), &env).unwrap();
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        for query in filtered_queries {
            assert_ne!(query.domain, "0.ubuntu.pool.ntp.org".to_owned());
            assert_ne!(query.domain, "1.ubuntu.pool.ntp.org".to_owned());
        }
    }
}
