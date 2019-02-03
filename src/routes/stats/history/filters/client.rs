// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Client Filter
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::queries,
    ftl::{FtlMemory, FtlQuery, ShmLockGuard},
    routes::stats::history::endpoints::HistoryParams,
    util::Error
};
use diesel::{prelude::*, sqlite::Sqlite};
use std::{collections::HashSet, iter};

/// Only show queries of the specified client
pub fn filter_client<'a>(
    queries_iter: Box<dyn Iterator<Item = &'a FtlQuery> + 'a>,
    params: &HistoryParams,
    ftl_memory: &FtlMemory,
    ftl_lock: &ShmLockGuard<'a>
) -> Result<Box<dyn Iterator<Item = &'a FtlQuery> + 'a>, Error> {
    if let Some(ref client_filter) = params.client {
        // Find the matching clients. If none are found, return an empty
        // iterator because no query can match the client requested
        let counters = ftl_memory.counters(ftl_lock)?;
        let strings = ftl_memory.strings(ftl_lock)?;
        let clients = ftl_memory.clients(ftl_lock)?;
        let client_ids: HashSet<usize> = clients
            .iter()
            .take(counters.total_clients as usize)
            .enumerate()
            .filter_map(|(i, client)| {
                let ip = client.get_ip(&strings);
                let name = client.get_name(&strings).unwrap_or_default();

                if ip.contains(client_filter) || name.contains(client_filter) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if !client_ids.is_empty() {
            Ok(Box::new(queries_iter.filter(move |query| {
                client_ids.contains(&(query.client_id as usize))
            })))
        } else {
            Ok(Box::new(iter::empty()))
        }
    } else {
        Ok(queries_iter)
    }
}

/// Only show queries of the specified client in database results
pub fn filter_client_db<'a>(
    db_query: queries::BoxedQuery<'a, Sqlite>,
    params: &HistoryParams
) -> queries::BoxedQuery<'a, Sqlite> {
    // Use the Diesel DSL of this table for easy querying
    use self::queries::dsl::*;

    if let Some(ref search_client) = params.client {
        db_query.filter(client.like(format!("%{}%", search_client)))
    } else {
        db_query
    }
}

#[cfg(test)]
mod test {
    use super::{filter_client, filter_client_db};
    use crate::{
        databases::ftl::connect_to_test_db,
        ftl::{FtlQuery, ShmLockGuard},
        routes::stats::history::{
            database::execute_query,
            endpoints::HistoryParams,
            testing::{test_memory, test_queries}
        }
    };
    use diesel::prelude::*;

    /// Only return queries from the specified client IP
    #[test]
    fn ip() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some("192.168.1.10".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries from the specified client IP. This test uses
    /// substring matching.
    #[test]
    fn ip_substring() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some(".10".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries from the specified client name
    #[test]
    fn name() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some("client1".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only return queries from the specified client name. This test uses
    /// substring matching.
    #[test]
    fn name_substring() {
        let queries = test_queries();
        let expected_queries = vec![&queries[0], &queries[1], &queries[2]];
        let filtered_queries: Vec<&FtlQuery> = filter_client(
            Box::new(queries.iter()),
            &HistoryParams {
                client: Some("t1".to_owned()),
                ..HistoryParams::default()
            },
            &test_memory(),
            &ShmLockGuard::Test
        )
        .unwrap()
        .collect();

        assert_eq!(filtered_queries, expected_queries);
    }

    /// Only queries with a client similar to the input are returned. This is a
    /// database filter.
    #[test]
    fn database() {
        use crate::databases::ftl::queries::dsl::*;

        let params = HistoryParams {
            client: Some("10.1".to_owned()),
            ..HistoryParams::default()
        };

        let db_query = filter_client_db(queries.into_boxed(), &params);
        let filtered_queries = execute_query(&connect_to_test_db(), db_query).unwrap();

        assert_eq!(filtered_queries.len(), 1);
        assert_eq!(filtered_queries[0].client, "10.1.1.1");
    }
}
