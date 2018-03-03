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

/// Get the query types usage over time
#[get("/stats/overTime/query_types")]
pub fn over_time_query_types(ftl: State<FtlConnectionType>) -> util::Reply {
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
