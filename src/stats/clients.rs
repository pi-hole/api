/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Clients Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::DecodeStringError;
use rmp::Marker;
use rocket::State;
use util;

/// Get the names of clients
// TODO: return only the names and IP addresses
#[get("/stats/clients")]
pub fn clients(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("client-names")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut client_data: Vec<(String, String, i32)> = Vec::new();

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

        client_data.push((name, ip, count));
    }

    util::reply_data(client_data)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestConfig, write_eom};

    #[test]
    fn test_clients() {
        let mut data = Vec::new();
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

        TestConfig::new()
            .endpoint("/admin/api/stats/clients")
            .ftl("client-names", data)
            .expect_json(
                json!({
                    "data": [
                        [
                            "client1",
                            "10.1.1.1",
                            30
                        ],
                        [
                            "",
                            "10.1.1.2",
                            20
                        ],
                        [
                            "client3",
                            "10.1.1.3",
                            10
                        ]
                    ],
                    "errors": []
                })
            )
            .test();
    }
}
