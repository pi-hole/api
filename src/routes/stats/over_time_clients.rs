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
use rocket::State;
use util::{Reply, ErrorKind, reply_data, reply_error};
use auth::User;
use routes::stats::clients::get_clients;

/// Get the client queries over time
#[get("/stats/overTime/clients")]
pub fn over_time_clients(_auth: User, ftl: State<FtlConnectionType>) -> Reply {
    let mut over_time = Vec::new();
    let clients = get_clients(&ftl)?;

    // Don't open another FTL connection until the connection from `get_clients` is done!
    // Otherwise FTL's global lock mechanism will cause a deadlock.
    let mut con = ftl.connect("ClientsoverTime")?;

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

        over_time.push(TimeStep { timestamp, data: step });
    }

    reply_data(json!({
        "over_time": over_time,
        "clients": clients
    }))
}

#[derive(Serialize)]
struct TimeStep {
    timestamp: i32,
    data: Vec<i32>
}

#[cfg(test)]
mod test {
    use rmp::encode;
    use testing::{TestBuilder, write_eom};

    #[test]
    fn test_over_time_clients() {
        let mut over_time = Vec::new();
        encode::write_i32(&mut over_time, 1520126228).unwrap();
        encode::write_i32(&mut over_time, 7).unwrap();
        encode::write_i32(&mut over_time, 3).unwrap();
        encode::write_i32(&mut over_time, -1).unwrap();
        encode::write_i32(&mut over_time, 1520126406).unwrap();
        encode::write_i32(&mut over_time, 6).unwrap();
        encode::write_i32(&mut over_time, 4).unwrap();
        encode::write_i32(&mut over_time, -1).unwrap();
        write_eom(&mut over_time);

        let mut clients = Vec::new();
        encode::write_str(&mut clients, "client1").unwrap();
        encode::write_str(&mut clients , "10.1.1.1").unwrap();
        encode::write_str(&mut clients , "").unwrap();
        encode::write_str(&mut clients , "10.1.1.2").unwrap();
        write_eom(&mut clients);

        TestBuilder::new()
            .endpoint("/admin/api/stats/overTime/clients")
            .ftl("ClientsoverTime", over_time)
            .ftl("client-names", clients)
            .expect_json(
                json!({
                    "over_time": [
                        {
                            "timestamp": 1520126228,
                            "data": [7, 3]
                        },
                        {
                            "timestamp": 1520126406,
                            "data": [6, 4]
                        }
                    ],
                    "clients": [
                        {
                            "name": "client1",
                            "ip": "10.1.1.1"
                        },
                        {
                            "name": "",
                            "ip": "10.1.1.2"
                        }
                    ]
                })
            )
            .test();
    }
}
