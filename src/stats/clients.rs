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
use auth::User;

/// Get the names of clients
#[get("/stats/clients")]
pub fn clients(_auth: User, ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("client-names")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut client_data = Vec::new();

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

        client_data.push(Client { name, ip });
    }

    util::reply_data(client_data)
}

#[derive(Serialize)]
struct Client {
    name: String,
    ip: String
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_clients() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_str(&mut data, "10.1.1.1").unwrap();
        encode::write_str(&mut data, "").unwrap();
        encode::write_str(&mut data, "10.1.1.2").unwrap();
        encode::write_str(&mut data, "client3").unwrap();
        encode::write_str(&mut data, "10.1.1.3").unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/clients")
            .ftl("client-names", data)
            .expect_json(
                json!([
                    {
                        "name": "client1",
                        "ip": "10.1.1.1"
                    },
                    {
                        "name": "",
                        "ip": "10.1.1.2"
                    },
                    {
                        "name": "client3",
                        "ip": "10.1.1.3"
                    }
                ])
            )
            .test();
    }
}
