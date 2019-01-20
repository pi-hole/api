// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Upstream Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    ftl::{FtlMemory, FtlQuery, FtlQueryStatus, ShmLockGuard},
    routes::stats::history::endpoints::HistoryParams,
    util::Error
};
use std::{collections::HashSet, iter};

/// Only show queries from the specified upstream
pub fn filter_upstream<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref upstream) = params.upstream {
        if upstream == "blocklist" {
            Ok(Box::new(queries_iter.filter(|query| match query.status {
                FtlQueryStatus::Gravity | FtlQueryStatus::Blacklist | FtlQueryStatus::Wildcard => {
                    true
                }
                _ => false
            })))
        } else if upstream == "cache" {
            Ok(Box::new(
                queries_iter.filter(|query| query.status == FtlQueryStatus::Cache)
            ))
        } else {
            // Find the matching upstreams. If none are found, return an empty
            // iterator because no query can match the upstream requested
            let counters = ftl_memory.counters(ftl_lock)?;
            let strings = ftl_memory.strings(ftl_lock)?;
            let upstreams = ftl_memory.upstreams(ftl_lock)?;
            let upstream_ids: HashSet<usize> = upstreams
                .iter()
                .take(counters.total_upstreams as usize)
                .enumerate()
                .filter_map(|(i, item)| {
                    let ip = item.get_ip(&strings);
                    let name = item.get_name(&strings).unwrap_or_default();

                    if ip.contains(upstream) || name.contains(upstream) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            if !upstream_ids.is_empty() {
                Ok(Box::new(queries_iter.filter(move |query| {
                    upstream_ids.contains(&(query.upstream_id as usize))
                })))
            } else {
                Ok(Box::new(iter::empty()))
            }
        }
    } else {
        Ok(queries_iter)
    }
}

#[cfg(test)]
mod test {
    use super::filter_upstream;
    use crate::{
        ftl::{FtlQuery, ShmLockGuard},
        routes::stats::history::{
            endpoints::HistoryParams,
            testing::{test_memory, test_queries}
        }
    };

    /// Only return queries with the specified upstream IP
    #[test]
    fn test_filter_upstream_ip() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("8.8.4.4".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified upstream IP. This test uses
    /// substring matching.
    #[test]
    fn test_filter_upstream_ip_substring() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("8.4.".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified upstream name
    #[test]
    fn test_filter_upstream_name() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("google-public-dns-b.google.com".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries with the specified upstream name. This test uses
    /// substring matching.
    #[test]
    fn test_filter_upstream_name_substring() {
        let queries = test_queries();
        let expected_queries = vec![&queries[7]];
        let filtered_queries: Vec<&FtlQuery> = filter_upstream(
            Box::new(queries.iter()),
            &HistoryParams {
                upstream: Some("b.google".to_owned()),
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
