// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Domain Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlQuery, ShmLockGuard},
    routes::stats::history::endpoints::HistoryParams,
    util::Error
};
use std::{collections::HashSet, iter};

/// Only show queries of the specified domain
pub fn filter_domain<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref domain_filter) = params.domain {
        // Find the matching domains. If none are found, return an empty
        // iterator because no query can match the domain requested
        let counters = ftl_memory.counters(ftl_lock)?;
        let strings = ftl_memory.strings(ftl_lock)?;
        let domains = ftl_memory.domains(ftl_lock)?;
        let domain_ids: HashSet<usize> = domains
            .iter()
            .take(counters.total_domains as usize)
            .enumerate()
            .filter_map(|(i, domain)| {
                if domain.get_domain(&strings).contains(domain_filter) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if !domain_ids.is_empty() {
            Ok(Box::new(queries_iter.filter(move |query| {
                domain_ids.contains(&(query.domain_id as usize))
            })))
        } else {
            Ok(Box::new(iter::empty()))
        }
    } else {
        Ok(queries_iter)
    }
}

#[cfg(test)]
mod test {
    use super::filter_domain;
    use crate::{
        ftl::{FtlQuery, ShmLockGuard},
        routes::stats::history::{
            endpoints::HistoryParams,
            testing::{test_memory, test_queries}
        }
    };

    /// Only return queries of the specified domain
    #[test]
    fn test_filter_domain() {
        let queries = test_queries();
        let expected_queries = vec![&queries[3]];
        let filtered_queries: Vec<&FtlQuery> = filter_domain(
            Box::new(queries.iter()),
            &HistoryParams {
                domain: Some("domain2.com".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries of the specified domain. This test uses substring
    /// matching.
    #[test]
    fn test_filter_domain_substring() {
        let queries = test_queries();
        let expected_queries = vec![&queries[3]];
        let filtered_queries: Vec<&FtlQuery> = filter_domain(
            Box::new(queries.iter()),
            &HistoryParams {
                domain: Some("2.c".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }
}
