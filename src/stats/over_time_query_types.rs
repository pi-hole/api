/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Query Types Over Time Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::ValueReadError;
use rmp::Marker;
use rocket::State;
use std::collections::HashMap;
use util;
use auth::Auth;

/// Get the query types usage over time
#[get("/stats/overTime/query_types")]
pub fn over_time_query_types(_auth: Auth, ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("QueryTypesoverTime")?;

    let mut over_time: HashMap<i32, (f32, f32)> = HashMap::new();

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

        let ipv4 = con.read_f32()?;
        let ipv6 = con.read_f32()?;

        over_time.insert(timestamp, (ipv4, ipv6));
    }

    util::reply_data(over_time)
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_over_time_query_types() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_f32(&mut data, 0.7).unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_f32(&mut data, 0.6).unwrap();
        encode::write_f32(&mut data, 0.4).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/query_types")
            .ftl("QueryTypesoverTime", data)
            .expect_json(
                json!({
                    "data": {
                        "1520126228": [
                            0.699999988079071,
                            0.30000001192092898
                        ],
                        "1520126406": [
                            0.6000000238418579,
                            0.4000000059604645
                        ]
                    },
                    "errors": []
                })
            )
            .test();
    }
}
