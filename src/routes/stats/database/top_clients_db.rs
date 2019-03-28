// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Top Clients Endpoint - DB Version
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    env::Env,
    ftl::BLOCKED_STATUSES,
    routes::{
        auth::User,
        stats::{
            check_privacy_level_top_clients,
            common::{get_excluded_clients, get_hidden_client_ip},
            database::{get_blocked_query_count, get_query_type_counts},
            top_clients::{TopClientItemReply, TopClientParams, TopClientsReply}
        }
    },
    settings::ValueType,
    util::{reply_result, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt};
use failure::ResultExt;
use rocket::{request::Form, State};

/// Get the top clients
#[get("/stats/database/top_clients?<from>&<until>&<params..>")]
pub fn top_clients_db(
    _auth: User,
    env: State<Env>,
    db: FtlDatabase,
    from: u64,
    until: u64,
    params: Form<TopClientParams>
) -> Reply {
    reply_result(top_clients_db_impl(
        &env,
        &db as &SqliteConnection,
        from,
        until,
        params.into_inner()
    ))
}

/// Get the top clients
fn top_clients_db_impl(
    env: &Env,
    db: &SqliteConnection,
    from: u64,
    until: u64,
    params: TopClientParams
) -> Result<TopClientsReply, Error> {
    // Resolve the parameters (the inactive param is ignored)
    let limit = params.limit.unwrap_or(10);
    let ascending = params.ascending.unwrap_or(false);
    let blocked = params.blocked.unwrap_or(false);

    let total_count = if blocked {
        get_blocked_query_count(db, from, until)?
    } else {
        // Total query count is the sum of all query type counts
        get_query_type_counts(db, from, until)?.values().sum()
    } as usize;

    // Check if the client details are private
    if let Some(reply) = check_privacy_level_top_clients(env, blocked, total_count)? {
        // We can not share any of the clients, so use the reply returned by the
        // function
        return Ok(reply);
    }

    let ignored_clients = get_ignored_clients(env)?;

    // Fetch the top clients and map into the reply structure
    let top_clients: Vec<TopClientItemReply> =
        execute_top_clients_query(db, from, until, ignored_clients, blocked, ascending, limit)?
            .into_iter()
            .map(|(client_identifier, count)| {
                if ValueType::Ipv4.is_valid(&client_identifier)
                    || ValueType::Ipv6.is_valid(&client_identifier)
                {
                    // If the identifier is an IP address, use it as the client IP
                    TopClientItemReply {
                        name: "".to_owned(),
                        ip: client_identifier,
                        count: count as usize
                    }
                } else {
                    // If the identifier is not an IP address, use it as the name
                    TopClientItemReply {
                        name: client_identifier,
                        ip: "".to_owned(),
                        count: count as usize
                    }
                }
            })
            .collect();

    // Output format changes when getting top blocked clients
    if blocked {
        Ok(TopClientsReply {
            top_clients,
            total_queries: None,
            blocked_queries: Some(total_count)
        })
    } else {
        Ok(TopClientsReply {
            top_clients,
            total_queries: Some(total_count),
            blocked_queries: None
        })
    }
}

/// Get the list of clients to ignore
fn get_ignored_clients(env: &Env) -> Result<Vec<String>, Error> {
    // Ignore clients excluded via SetupVars
    let mut ignored_clients = get_excluded_clients(env)?;

    // Ignore the hidden client IP (due to privacy level)
    ignored_clients.push(get_hidden_client_ip().to_owned());

    Ok(ignored_clients)
}

/// Create and execute the database query to retrieve the top client details.
/// The returned Vec contains each client's identifier and count, sorted and
/// ordered according to the parameters.
fn execute_top_clients_query(
    db: &SqliteConnection,
    from: u64,
    until: u64,
    ignored_clients: Vec<String>,
    blocked: bool,
    ascending: bool,
    limit: usize
) -> Result<Vec<(String, i64)>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // Create query
    let db_query = queries
        .select((client, sql::<BigInt>("COUNT(*)")))
        // Only consider queries in the time interval
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.le(until as i32))
        // Filter out ignored clients
        .filter(client.ne_all(ignored_clients))
        // Group queries by client
        .group_by(client)
        // Take into account the limit
        .limit(limit as i64)
        // Box the query so we can conditionally modify it
        .into_boxed();

    // Set the sort order
    let db_query = if ascending {
        db_query.order((sql::<BigInt>("COUNT(*)").asc(), client))
    } else {
        db_query.order((sql::<BigInt>("COUNT(*)").desc(), client))
    };

    // Filter by status
    let db_query = if blocked {
        db_query.filter(status.eq_any(&BLOCKED_STATUSES))
    } else {
        // If not blocked, use all queries
        db_query
    };

    // Execute query
    Ok(db_query
        .load::<(String, i64)>(db)
        .context(ErrorKind::FtlDatabase)?)
}

#[cfg(test)]
mod test {
    use super::top_clients_db_impl;
    use crate::{
        databases::ftl::connect_to_test_db,
        env::{Config, Env, PiholeFile},
        routes::stats::top_clients::{TopClientItemReply, TopClientParams, TopClientsReply},
        testing::TestEnvBuilder
    };
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 0;
    const UNTIL_TIMESTAMP: u64 = 177_180;

    /// The default behavior lists all clients in descending order
    #[test]
    fn default_params() {
        let expected = TopClientsReply {
            top_clients: vec![
                TopClientItemReply {
                    name: "".to_owned(),
                    ip: "127.0.0.1".to_owned(),
                    count: 93
                },
                TopClientItemReply {
                    name: "".to_owned(),
                    ip: "10.1.1.1".to_owned(),
                    count: 1
                },
            ],
            total_queries: Some(94),
            blocked_queries: None
        };

        let db = connect_to_test_db();
        let env = Env::Test(Config::default(), HashMap::new());
        let params = TopClientParams::default();
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// Show only blocked clients
    #[test]
    fn blocked_clients() {
        // There are no blocked clients in the database
        let expected = TopClientsReply {
            top_clients: Vec::new(),
            total_queries: None,
            blocked_queries: Some(0)
        };

        let db = connect_to_test_db();
        let env = Env::Test(Config::default(), HashMap::new());
        let params = TopClientParams {
            blocked: Some(true),
            ..TopClientParams::default()
        };
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// The number of clients shown is <= the limit
    #[test]
    fn limit() {
        let expected = TopClientsReply {
            top_clients: vec![TopClientItemReply {
                name: "".to_owned(),
                ip: "127.0.0.1".to_owned(),
                count: 93
            }],
            total_queries: Some(94),
            blocked_queries: None
        };

        let db = connect_to_test_db();
        let env = Env::Test(Config::default(), HashMap::new());
        let params = TopClientParams {
            limit: Some(1),
            ..TopClientParams::default()
        };
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// Same as the default behavior but in ascending order
    #[test]
    fn ascending() {
        let expected = TopClientsReply {
            top_clients: vec![
                TopClientItemReply {
                    name: "".to_owned(),
                    ip: "10.1.1.1".to_owned(),
                    count: 1
                },
                TopClientItemReply {
                    name: "".to_owned(),
                    ip: "127.0.0.1".to_owned(),
                    count: 93
                },
            ],
            total_queries: Some(94),
            blocked_queries: None
        };

        let db = connect_to_test_db();
        let env = Env::Test(Config::default(), HashMap::new());
        let params = TopClientParams {
            ascending: Some(true),
            ..TopClientParams::default()
        };
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// Privacy level 2 does not show any clients
    #[test]
    fn privacy() {
        let expected = TopClientsReply {
            top_clients: Vec::new(),
            total_queries: Some(94),
            blocked_queries: None
        };

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .build();
        let params = TopClientParams::default();
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// For top blocked clients, privacy level 2 does not show any clients and
    /// has a `blocked_queries` key instead of a `total_queries` key
    #[test]
    fn privacy_blocked() {
        let expected = TopClientsReply {
            top_clients: Vec::new(),
            total_queries: None,
            blocked_queries: Some(0)
        };

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::FtlConfig, "PRIVACYLEVEL=2")
            .build();
        let params = TopClientParams {
            blocked: Some(true),
            ..TopClientParams::default()
        };
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }

    /// Excluded clients are not shown
    #[test]
    fn excluded_clients() {
        let expected = TopClientsReply {
            top_clients: vec![TopClientItemReply {
                name: "".to_owned(),
                ip: "10.1.1.1".to_owned(),
                count: 1
            }],
            total_queries: Some(94),
            blocked_queries: None
        };

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=127.0.0.1")
            .build();
        let params = TopClientParams::default();
        let actual =
            top_clients_db_impl(&env, &db, FROM_TIMESTAMP, UNTIL_TIMESTAMP, params).unwrap();

        assert_eq!(actual, expected);
    }
}
