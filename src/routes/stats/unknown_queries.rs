// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Unknown Queries Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use auth::User;
use ftl::FtlConnectionType;
use rocket::State;
use util::{reply_data, reply_error, ErrorKind, Reply};

/// Represents a query returned in `/stats/unknown_queries`
#[derive(Serialize)]
struct UnknownQuery(i32, i32, String, String, String, u8, bool);

/// Get all unknown queries
#[get("/stats/unknown_queries")]
pub fn unknown_queries(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    let mut con = ftl.connect("unknown")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];
    let mut queries: Vec<UnknownQuery> = Vec::new();

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

        // Read the rest of the data
        let id = con.read_i32()?;
        let query_type = con.read_str(&mut str_buffer)?.to_owned();
        let domain = con.read_str(&mut str_buffer)?.to_owned();
        let client = con.read_str(&mut str_buffer)?.to_owned();
        let status = con.read_u8()?;
        let complete = con.read_bool()?;

        queries.push(UnknownQuery(
            timestamp, id, query_type, domain, client, status, complete,
        ));
    }

    reply_data(queries)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{write_eom, TestBuilder};

    #[test]
    fn test_unknown_queries() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_i32(&mut data, 0).unwrap();
        encode::write_str(&mut data, "IPv4").unwrap();
        encode::write_str(&mut data, "example.com").unwrap();
        encode::write_str(&mut data, "client1").unwrap();
        encode::write_u8(&mut data, 2).unwrap();
        encode::write_bool(&mut data, false).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_i32(&mut data, 1).unwrap();
        encode::write_str(&mut data, "IPv6").unwrap();
        encode::write_str(&mut data, "doubleclick.com").unwrap();
        encode::write_str(&mut data, "client2").unwrap();
        encode::write_u8(&mut data, 1).unwrap();
        encode::write_bool(&mut data, true).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/unknown_queries")
            .ftl("unknown", data)
            .expect_json(json!([
                [1520126228, 0, "IPv4", "example.com", "client1", 2, false],
                [1520126406, 1, "IPv6", "doubleclick.com", "client2", 1, true]
            ]))
            .test();
    }
}
