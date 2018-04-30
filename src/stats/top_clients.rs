/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Top Clients Endpoints
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::DecodeStringError;
use rmp::Marker;
use rocket::State;
use util;
use auth::User;

/// Get the top clients with default parameters
#[get("/stats/top_clients")]
pub fn top_clients(_auth: User, ftl: State<FtlConnectionType>) -> util::Reply {
    get_top_clients(&ftl, TopClientParams::default())
}

/// Get the top clients with specified parameters
#[get("/stats/top_clients?<params>")]
pub fn top_clients_params(_auth: User, ftl: State<FtlConnectionType>, params: TopClientParams) -> util::Reply {
    get_top_clients(&ftl, params)
}

/// Represents the possible GET parameters on `/stats/top_clients`
#[derive(FromForm)]
pub struct TopClientParams {
    limit: Option<usize>,
    inactive: Option<bool>,
    ascending: Option<bool>
}

impl Default for TopClientParams {
    /// The default parameters of top_clients requests
    fn default() -> Self {
        TopClientParams {
            limit: Some(10),
            inactive: Some(false),
            ascending: Some(false)
        }
    }
}

#[derive(Serialize)]
struct Client {
    name: String,
    ip: String,
    count: i32
}

fn get_top_clients(ftl: &FtlConnectionType, params: TopClientParams) -> util::Reply {
    let default_limit: usize = 10;

    // Create the command to send to FTL
    let command = format!(
        "top-clients ({}){}{}",
        params.limit.unwrap_or(default_limit),
        if params.inactive.unwrap_or(false) { " withzero" } else { "" },
        if params.ascending.unwrap_or(false) { " asc" } else { "" }
    );

    // Connect to FTL
    let mut con = ftl.connect(&command)?;

    // Get the total number of queries
    let total_queries = con.read_i32()?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    // Store the hostname -> number data here
    let mut top_clients = Vec::new();

    loop {
        // Get the hostname, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_owned(),
            Err(e) => {
                // Check if we received the EOM
                if let DecodeStringError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        let ip = con.read_str(&mut str_buffer)?.to_owned();
        let count = con.read_i32()?;

        top_clients.push(Client { name, ip, count });
    }

    util::reply_data(json!({
        "top_clients": top_clients,
        "total_queries": total_queries
    }))
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_top_clients() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 100).unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_str(&mut data, "10.1.1.1").unwrap();
        encode::write_i32(&mut data, 30).unwrap();
        encode::write_str(&mut data, "").unwrap();
        encode::write_str(&mut data, "10.1.1.2").unwrap();
        encode::write_i32(&mut data, 20).unwrap();
        encode::write_str(&mut data, "client3").unwrap();
        encode::write_str(&mut data, "10.1.1.3").unwrap();
        encode::write_i32(&mut data, 10).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients")
            .ftl("top-clients (10)", data)
            .expect_json(
                json!({
                    "top_clients": [
                        {
                            "name": "client1",
                            "ip": "10.1.1.1",
                            "count": 30
                        },
                        {
                            "name": "",
                            "ip": "10.1.1.2",
                            "count": 20
                        },
                        {
                            "name": "client3",
                            "ip": "10.1.1.3",
                            "count": 10
                        }
                    ],
                    "total_queries": 100
                })
            )
            .test();
    }

    #[test]
    fn test_top_clients_limit() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 100).unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_str(&mut data, "10.1.1.1").unwrap();
        encode::write_i32(&mut data, 30).unwrap();
        encode::write_str(&mut data, "").unwrap();
        encode::write_str(&mut data, "10.1.1.2").unwrap();
        encode::write_i32(&mut data, 20).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/top_clients?limit=2")
            .ftl("top-clients (2)", data)
            .expect_json(
                json!({
                    "top_clients": [
                        {
                            "name": "client1",
                            "ip": "10.1.1.1",
                            "count": 30
                        },
                        {
                            "name": "",
                            "ip": "10.1.1.2",
                            "count": 20
                        }
                    ],
                    "total_queries": 100
                })
            )
            .test();
    }
}
