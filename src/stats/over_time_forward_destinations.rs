/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Forward Destinations Over Time Endpoint
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

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_over_time_forward_destinations() {
        let mut data = Vec::new();
        encode::write_i32(&mut data, 3).unwrap();
        encode::write_str(&mut data, "google-dns-alt").unwrap();
        encode::write_str(&mut data, "8.8.4.4").unwrap();
        encode::write_str(&mut data, "google-dns").unwrap();
        encode::write_str(&mut data, "8.8.8.8").unwrap();
        encode::write_str(&mut data, "local").unwrap();
        encode::write_str(&mut data, "local").unwrap();
        encode::write_i32(&mut data, 1520126228).unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        encode::write_f32(&mut data, 0.4).unwrap();
        encode::write_i32(&mut data, 1520126406).unwrap();
        encode::write_f32(&mut data, 0.5).unwrap();
        encode::write_f32(&mut data, 0.2).unwrap();
        encode::write_f32(&mut data, 0.3).unwrap();
        write_eom(&mut data);

        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/forward_destinations")
            .ftl("ForwardedoverTime", data)
            .expect_json(
                json!({
                    "data": {
                        "forward_destinations": [
                            "google-dns-alt|8.8.4.4",
                            "google-dns|8.8.8.8",
                            "local|local"
                        ],
                        "over_time": {
                            "1520126228": [
                                0.30000001192092898,
                                0.30000001192092898,
                                0.4000000059604645
                            ],
                            "1520126406": [
                                0.5,
                                0.20000000298023225,
                                0.30000001192092898
                            ]
                        }
                    },
                    "errors": []
                })
            )
            .test();
    }
}
