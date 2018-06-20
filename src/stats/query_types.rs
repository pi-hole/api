/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query Types Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rocket::State;
use util::{Reply, ErrorKind, reply_data, reply_error};
use auth::User;

/// Get the query types
#[get("/stats/query_types")]
pub fn query_types(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    let mut con = ftl.connect("querytypes")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut query_types = Vec::new();

    loop {
        // Read in the name, unless we are at the end of the list
        let name = match con.read_str(&mut str_buffer) {
            Ok(name) => name.to_string(),
            Err(e) => {
                // Check if we received the EO
                // Check if we received the EOM
                if e.kind() == ErrorKind::FtlEomError {
                    break;
                }

                // Unknown read error
                return reply_error(e);
            }
        };

        let percent = con.read_f32()?;

        query_types.push(QueryType { name, percent });
    }

    reply_data(query_types)
}

#[derive(Serialize)]
struct QueryType {
    name: String,
    percent: f32
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_query_types() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "A (IPv4)").unwrap();
        encode::write_f32(&mut data, 0.1).unwrap();
        encode::write_str(&mut data, "AAAA (IPv6)").unwrap();
        encode::write_f32(&mut data, 0.2).unwrap();
        encode::write_str(&mut data, "ANY").unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_str(&mut data, "SRV").unwrap();
        encode::write_f32(&mut data, 0.4).unwrap();
        encode::write_str(&mut data, "SOA").unwrap();
        encode::write_f32(&mut data, 0.5).unwrap();
        encode::write_str(&mut data, "PTR").unwrap();
        encode::write_f32(&mut data, 0.6).unwrap();
        encode::write_str(&mut data, "TXT").unwrap();
        encode::write_f32(&mut data, 0.7).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/query_types")
            .ftl("querytypes", data)
            .expect_json(
                json!([
                    {
                        "name": "A (IPv4)",
                        "percent": 0.10000000149011612
                    },
                    {
                        "name": "AAAA (IPv6)",
                        "percent": 0.20000000298023225
                    },
                    {
                        "name": "ANY",
                        "percent": 0.30000001192092898
                    },
                    {
                        "name": "SRV",
                        "percent": 0.4000000059604645
                    },
                    {
                        "name": "SOA",
                        "percent": 0.5
                    },
                    {
                        "name": "PTR",
                        "percent": 0.6000000238418579
                    },
                    {
                        "name": "TXT",
                        "percent": 0.699999988079071
                    }
                ])
            )
            .test();
    }
}
