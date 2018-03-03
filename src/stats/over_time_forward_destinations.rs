/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Forward Destinations Over Time API Endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use ftl::FtlConnectionType;
use rmp::decode::ValueReadError;
use rmp::Marker;
use rocket::State;
use std::collections::HashMap;
use util;

/// Get the forward destination usage over time
#[get("/stats/overTime/forward_destinations")]
pub fn over_time_forward_destinations(ftl: State<FtlConnectionType>) -> util::Reply {
    let mut con = ftl.connect("ForwardedoverTime")?;

    // Create a 4KiB string buffer
    let mut str_buffer = [0u8; 4096];

    // Read in the number of forward destinations
    let forward_dest_num = con.read_i32()? as usize;

    // Create the data structures to store the data in
    let mut forward_data: Vec<String> = Vec::with_capacity(forward_dest_num);
    let mut over_time: HashMap<i32, Vec<f32>> = HashMap::new();

    // Read in forward destination names and IPs
    for _ in 0..forward_dest_num {
        let name = con.read_str(&mut str_buffer)?.to_owned();
        let ip = con.read_str(&mut str_buffer)?.to_owned();

        forward_data.push(
            // The key will be `hostname|IP` if the hostname exists, otherwise just the IP address
            if name.is_empty() {
                ip
            } else {
                format!("{}|{}", name, ip)
            }
        );
    }

    // Get the over time data
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

        // Create a new step in the graph (stores the value of each destination usage at that time)
        let mut step = Vec::with_capacity(forward_dest_num);

        // Read in the forward destination usage
        for _ in 0..forward_dest_num {
            step.push(con.read_f32()?);
        }

        over_time.insert(timestamp, step);
    }

    util::reply_data(json!({
        "forward_destinations": forward_data,
        "over_time": over_time
    }))
}
