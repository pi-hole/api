// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Query History Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use ftl::FtlConnectionType;
use rocket::State;
use util::{Reply, ErrorKind, reply_data, reply_error};
use auth::User;

/// Get the entire query history (as stored in FTL)
#[get("/stats/history")]
pub fn history(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    get_history(&ftl, "getallqueries", None)
}

/// Get the query history according to the specified parameters
#[get("/stats/history?<params>")]
pub fn history_params(
    _auth: User,
    ftl: State<FtlConnectionType>,
    params: HistoryParams
) -> Reply {
    let limit = params.limit;
    let command = match params {
        // Get the query history within the specified timespan
        HistoryParams {
            from: Some(from),
            until: Some(until),
            domain: None,
            client: None,
            ..
        } => {
            format!("getallqueries-time {} {}", from, until)
        },
        // Get the query history for the specified domain
        HistoryParams {
            from: None,
            until: None,
            domain: Some(domain),
            client: None,
            ..
        } => {
            format!("getallqueries-domain {}", domain)
        },
        // Get the query history for the specified client
        HistoryParams {
            from: None,
            until: None,
            domain: None,
            client: Some(client),
            ..
        } => {
            format!("getallqueries-client {}", client)
        },
        // FTL can't handle mixed input
        _ => return reply_error(ErrorKind::BadRequest)
    };

    get_history(
        &ftl,
        &command,
        limit
    )
}

/// Represents a query returned in `/stats/history`
#[derive(Serialize)]
struct Query(i32, String, String, String, u8, u8);

/// Represents the possible GET parameters on `/stats/history`
#[derive(FromForm)]
pub struct HistoryParams {
    from: Option<u64>,
    until: Option<u64>,
    domain: Option<String>,
    client: Option<String>,
    limit: Option<usize>
}

/// Get the query history according to the specified command
fn get_history(ftl: &FtlConnectionType, command: &str, limit: Option<usize>) -> Reply {
    let full_command = if let Some(num) = limit {
        format!("{} ({})", command, num)
    } else {
        command.to_owned()
    };

    let mut con = ftl.connect(&full_command)?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut history: Vec<Query> = Vec::new();

    loop {
        // Get the timestamp, unless we are at the end of the list
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
                // Check if we received the EOM
                if e.kind() == ErrorKind::FtlEomError {
                    break;
                }

                // Unknown read error
                return reply_error(e);
            }
        };

        // Get the rest of the query data
        let query_type = con.read_str(&mut str_buffer)?.to_owned();
        let domain = con.read_str(&mut str_buffer)?.to_owned();
        let client = con.read_str(&mut str_buffer)?.to_owned();
        let status = con.read_u8()?;
        let dnssec = con.read_u8()?;

        history.push(Query(timestamp, query_type, domain, client, status, dnssec));
    }

    reply_data(history)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_history() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_str(&mut data, "IPv6").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "client2").unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history")
            .ftl("getallqueries", data)
            .expect_json(
                json!([
                    [
                        1520126228,
                        "IPv4",
                        "example.com",
                        "client1",
                        2,
                        1
                    ],
                    [
                        1520126406,
                        "IPv6",
                        "doubleclick.com",
                        "client2",
                        1,
                        1
                    ]
                ])
            )
            .test();
    }

    #[test]
    fn test_history_timespan() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_str(&mut data, "IPv6").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "client2").unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?from=1520126228&until=1520126406")
            .ftl("getallqueries-time 1520126228 1520126406", data)
            .expect_json(
                json!([
                    [
                        1520126228,
                        "IPv4",
                        "example.com",
                        "client1",
                        2,
                        1
                    ],
                    [
                        1520126406,
                        "IPv6",
                        "doubleclick.com",
                        "client2",
                        1,
                        1
                    ]
                ])
            )
            .test();
    }

    #[test]
    fn test_history_domain() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?domain=example.com")
            .ftl("getallqueries-domain example.com", data)
            .expect_json(
                json!([
                    [
                        1520126228,
                        "IPv4",
                        "example.com",
                        "client1",
                        2,
                        1
                    ]
                ])
            )
            .test();
    }

    #[test]
    fn test_history_client() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/history?client=client1")
            .ftl("getallqueries-client client1", data)
            .expect_json(
                json!([
                    [
                        1520126228,
                        "IPv4",
                        "example.com",
                        "client1",
                        2,
                        1
                    ]
                ])
            )
            .test();
    }
}
