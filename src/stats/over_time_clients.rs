/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Clients Over Time Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::ValueReadError;
use rmp::Marker;
use rocket::State;
use std::collections::HashMap;
use util;

/// Get the client queries over time
#[get("/stats/overTime/clients")]
pub fn over_time_clients(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("ClientsoverTime")?;

    let mut over_time: HashMap<i32, Vec<i32>> = HashMap::new();

    loop {
        // Get the timestamp, unless we are at the end of the list
        let timestamp = match con.read_i32() {
            Ok(timestamp) => timestamp,
            Err(e) => {
                // Check if we received the EOM
                if let ValueReadError::TypeMismatch(marker) = e {
                    if marker == Marker::Reserved {
                        // Received EOM
                        break;
                    }
                }

                // Unknown read error
                return util::reply_error(util::Error::Unknown);
            }
        };

        // Create a new step in the graph (stores the value of each client usage at that time)
        let mut step = Vec::new();

        // Get all the data for this step
        loop {
            let client = con.read_i32()?;

            // Marker for the end of this step
            if client == -1 {
                break;
            }

            step.push(client);
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(over_time)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_over_time_clients() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_i32(&mut data, 7).unwrap();
        encode::write_i32(&mut data, 3).unwrap();
        encode::write_i32(&mut data, -1).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_i32(&mut data, 6).unwrap();
        encode::write_i32(&mut data, 4).unwrap();
        encode::write_i32(&mut data, -1).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/clients")
            .ftl("ClientsoverTime", data)
            .expect_json(
                json!({
                    "1520126228": [7, 3],
                    "1520126406": [6, 4]
                })
            )
            .test();
    }
}
