// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Clients Over Time Database Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    databases::ftl::FtlDatabase,
    env::Env,
    ftl::ClientReply,
    routes::{
        auth::User,
        stats::{
            common::{get_excluded_clients, HIDDEN_CLIENT},
            database::over_time_history_db::align_from_until,
            over_time_clients::{OverTimeClientItem, OverTimeClients}
        }
    },
    settings::ValueType,
    util::{reply_result, Error, ErrorKind, Reply}
};
use diesel::{dsl::sql, prelude::*, sql_types::BigInt, SqliteConnection};
use failure::ResultExt;
use rocket::State;
use std::collections::HashMap;

/// Get the clients queries over time data from the database
#[get("/stats/database/overTime/clients?<from>&<until>&<interval>")]
pub fn over_time_clients_db(
    from: u64,
    until: u64,
    interval: Option<usize>,
    _auth: User,
    db: FtlDatabase,
    env: State<Env>
) -> Reply {
    reply_result(over_time_clients_db_impl(
        from,
        until,
        interval.unwrap_or(600),
        &db as &SqliteConnection,
        &env
    ))
}

/// Get the clients queries over time data from the database
fn over_time_clients_db_impl(
    from: u64,
    until: u64,
    interval: usize,
    db: &SqliteConnection,
    env: &Env
) -> Result<OverTimeClients, Error> {
    let (from, until) = align_from_until(from, until, interval as u64)?;

    // Load the clients (names or IP addresses)
    let client_identifiers = get_client_identifiers(from, until, db, env)?;

    // Build the timestamp -> client query data map
    let mut over_time_data: HashMap<u64, Vec<usize>> = (from..until)
        .step_by(interval)
        .map(|timestamp| (timestamp, vec![0; client_identifiers.len()]))
        .collect();

    for (client_index, client_identifier) in client_identifiers.iter().enumerate() {
        // For each client, get the overTime data
        let client_over_time = get_client_over_time(from, until, interval, client_identifier, db)?;

        // Add the client's data to the overTime map
        for (timestamp, value) in client_over_time {
            let client_data = over_time_data.get_mut(&(timestamp as u64)).unwrap();
            client_data[client_index] = value as usize;
        }
    }

    // Convert the overTime data into the output format
    let mut over_time: Vec<OverTimeClientItem> = over_time_data
        .into_iter()
        .map(|(timestamp, data)| OverTimeClientItem {
            // Display the timestamps as centered in the overTime slot interval
            timestamp: timestamp + (interval / 2) as u64,
            data
        })
        .collect();

    // Make sure we return the overTime data in sorted order (by timestamp)
    over_time.sort();

    // Convert the client identifiers into the output format
    let clients = client_identifiers
        .into_iter()
        .map(|client_identifier| {
            if ValueType::IPv4.is_valid(&client_identifier)
                || ValueType::IPv6.is_valid(&client_identifier)
            {
                // If the identifier is an IP address, use it as the client IP
                ClientReply {
                    name: "".to_owned(),
                    ip: client_identifier
                }
            } else {
                // If the identifier is not an IP address, use it as the name
                ClientReply {
                    name: client_identifier,
                    ip: "".to_owned()
                }
            }
        })
        .collect();

    Ok(OverTimeClients { over_time, clients })
}

/// Get clients which made queries during the interval. The values may be either
/// hostnames or IP addresses
fn get_client_identifiers(
    from: u64,
    until: u64,
    db: &SqliteConnection,
    env: &Env
) -> Result<Vec<String>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // Find clients which should not be used
    let mut ignored_clients = get_excluded_clients(env)?;
    ignored_clients.push(HIDDEN_CLIENT.to_owned());

    let client_identifiers = queries
        .select(client)
        .distinct()
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.lt(until as i32))
        .filter(client.ne_all(ignored_clients))
        .load(db)
        .context(ErrorKind::FtlDatabase)?;

    Ok(client_identifiers)
}

/// Get the overTime data for the client in the specified interval
fn get_client_over_time(
    from: u64,
    until: u64,
    interval: usize,
    client_identifier: &str,
    db: &SqliteConnection
) -> Result<HashMap<i32, i64>, Error> {
    use crate::databases::ftl::queries::dsl::*;

    // SQL snippet for calculating the interval timestamp of the query
    let interval_sql = sql(&format!(
        "(timestamp / {interval}) * {interval}",
        interval = interval
    ));

    // Create SQL query
    let sql_query = queries
        .select((&interval_sql, sql::<BigInt>("COUNT(*)")))
        .filter(client.eq(client_identifier))
        .filter(timestamp.ge(from as i32))
        .filter(timestamp.lt(until as i32))
        .group_by(&interval_sql);

    // Execute SQL query
    Ok(sql_query
        .load(db)
        .context(ErrorKind::FtlDatabase)?
        // Convert to HashMap
        .into_iter()
        .collect())
}

#[cfg(test)]
mod test {
    use super::{get_client_identifiers, get_client_over_time, over_time_clients_db_impl};
    use crate::{
        databases::ftl::connect_to_test_db,
        env::PiholeFile,
        ftl::ClientReply,
        routes::stats::over_time_clients::{OverTimeClientItem, OverTimeClients},
        testing::TestEnvBuilder
    };
    use std::collections::HashMap;

    const FROM_TIMESTAMP: u64 = 164_400;
    const UNTIL_TIMESTAMP: u64 = 165_600;
    const INTERVAL: usize = 600;

    /// Verify the over time data is retrieved correctly
    #[test]
    fn over_time_clients_impl() {
        let expected = OverTimeClients {
            clients: vec![
                ClientReply {
                    name: "".to_owned(),
                    ip: "127.0.0.1".to_owned()
                },
                ClientReply {
                    name: "".to_owned(),
                    ip: "10.1.1.1".to_owned()
                },
            ],
            over_time: vec![
                OverTimeClientItem {
                    timestamp: 164_700,
                    data: vec![25, 1]
                },
                OverTimeClientItem {
                    timestamp: 165_300,
                    data: vec![7, 0]
                },
                OverTimeClientItem {
                    timestamp: 165_900,
                    data: vec![0, 0]
                },
            ]
        };

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "")
            .build();
        let actual =
            over_time_clients_db_impl(FROM_TIMESTAMP, UNTIL_TIMESTAMP, INTERVAL, &db, &env)
                .unwrap();

        assert_eq!(actual, expected);
    }

    /// The client identifiers stored in the database are all retrieved for a
    /// specified interval
    #[test]
    fn client_identifiers() {
        let expected = vec!["127.0.0.1".to_owned(), "10.1.1.1".to_owned()];

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "")
            .build();
        let actual = get_client_identifiers(FROM_TIMESTAMP, UNTIL_TIMESTAMP, &db, &env).unwrap();

        assert_eq!(actual, expected);
    }

    /// If a client is excluded, it is not returned
    #[test]
    fn client_identifiers_excluded() {
        let expected = vec!["127.0.0.1".to_owned()];

        let db = connect_to_test_db();
        let env = TestEnvBuilder::new()
            .file(PiholeFile::SetupVars, "API_EXCLUDE_CLIENTS=10.1.1.1")
            .build();
        let actual = get_client_identifiers(FROM_TIMESTAMP, UNTIL_TIMESTAMP, &db, &env).unwrap();

        assert_eq!(actual, expected);
    }

    /// The client-specific overTime data is retrieved correctly for a specified
    /// time interval
    #[test]
    fn client_over_time() {
        let mut expected: HashMap<i32, i64> = HashMap::new();
        expected.insert(164_400, 25);
        expected.insert(165_000, 7);

        let db = connect_to_test_db();
        let actual =
            get_client_over_time(FROM_TIMESTAMP, UNTIL_TIMESTAMP, INTERVAL, "127.0.0.1", &db)
                .unwrap();

        assert_eq!(actual, expected);
    }
}
